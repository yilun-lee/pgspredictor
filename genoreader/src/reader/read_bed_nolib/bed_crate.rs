use std::{
    fmt::Debug,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    ops::{Div, Sub},
    path::PathBuf,
};

use anyinput::anyinput;
use ndarray as nd;
use rayon::iter::{ParallelBridge, ParallelIterator};

use super::bed_error::{BedError, BedErrorPlus};

const BED_FILE_MAGIC1: u8 = 0x6C; // 0b01101100 or 'l' (lowercase 'L')
const BED_FILE_MAGIC2: u8 = 0x1B; // 0b00011011 or <esc>
const CB_HEADER_U64: u64 = 3;
const CB_HEADER_USIZE: usize = 3;

pub trait BedVal: Copy + Default + From<i8> + Debug + Sync + Send + Missing + PartialEq {}
impl<T> BedVal for T where T: Copy + Default + From<i8> + Debug + Sync + Send + Missing + PartialEq {}

#[allow(clippy::too_many_arguments)]
#[allow(unused_variables)]
#[anyinput]
pub fn read_no_alloc<TVal: BedVal>(
    path: AnyPath,
    iid_count: usize,
    sid_count: usize,
    is_a1_counted: bool,
    iid_index: &[isize],
    sid_index: &[isize],
    missing_value: TVal,
    num_threads: usize,
    val: &mut nd::ArrayViewMut2<'_, TVal>, /* mutable slices additionally allow to modify
                                            * elements. But slices cannot grow - they are just a
                                            * view into some vector. */
) -> Result<(), BedErrorPlus> {
    let (buf_reader, bytes_vector) = open_and_check(path)?;

    match bytes_vector[2] {
        0 => {
            // We swap 'iid' and 'sid' and then reverse the axes.
            let mut val_t = val.view_mut().reversed_axes();
            internal_read_no_alloc(
                buf_reader,
                path,
                sid_count,
                iid_count,
                is_a1_counted,
                sid_index,
                iid_index,
                missing_value,
                &mut val_t,
            )?
        }
        1 => internal_read_no_alloc(
            buf_reader,
            path,
            iid_count,
            sid_count,
            is_a1_counted,
            iid_index,
            sid_index,
            missing_value,
            val,
        )?,
        _ => return Err(BedError::BadMode(path_ref_to_string(path)).into()),
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[anyinput]
fn internal_read_no_alloc<TVal: BedVal>(
    mut buf_reader: BufReader<File>,
    path: AnyPath,
    in_iid_count: usize,
    in_sid_count: usize,
    is_a1_counted: bool,
    iid_index: &[isize],
    sid_index: &[isize],
    missing_value: TVal,
    out_val: &mut nd::ArrayViewMut2<'_, TVal>, /* mutable slices additionally allow to modify
                                                * elements. But slices cannot grow - they are
                                                * just a view into some vector. */
) -> Result<(), BedErrorPlus> {
    // Check the file length

    let (in_iid_count_div4, in_iid_count_div4_u64) =
        try_div_4(in_iid_count, in_sid_count, CB_HEADER_U64)?;
    // "as" and math is safe because of early checks
    let file_len = buf_reader.seek(SeekFrom::End(0))?;
    let file_len2 = in_iid_count_div4_u64 * (in_sid_count as u64) + CB_HEADER_U64;
    if file_len != file_len2 {
        return Err(BedError::IllFormed(path_ref_to_string(path)).into());
    }

    // Check and precompute for each iid_index
    let (i_div_4_array, i_mod_4_times_2_array) =
        check_and_precompute_iid_index(in_iid_count, iid_index)?;

    // Check and compute work for each sid_index

    let from_two_bits_to_value = set_up_two_bits_to_value(is_a1_counted, missing_value);
    let lower_sid_count = -(in_sid_count as isize);
    let upper_sid_count: isize = (in_sid_count as isize) - 1;
    // See https://morestina.net/blog/1432/parallel-stream-processing-with-rayon
    // Possible optimization: We could try to read only the iid info needed
    // Possible optimization: We could read snp in their input order instead of
    // their output order
    sid_index
        .iter()
        .map(|in_sid_i_signed| {
            // Turn signed sid_index into unsigned sid_index (or error)
            let in_sid_i = if (0..=upper_sid_count).contains(in_sid_i_signed) {
                *in_sid_i_signed as u64
            } else if (lower_sid_count..=-1).contains(in_sid_i_signed) {
                (in_sid_count - ((-in_sid_i_signed) as usize)) as u64
            } else {
                return Err(BedErrorPlus::BedError(BedError::SidIndexTooBig(
                    *in_sid_i_signed,
                )));
            };

            // Read the iid info for one snp from the disk
            let mut bytes_vector: Vec<u8> = vec![0; in_iid_count_div4];
            let pos: u64 = in_sid_i * in_iid_count_div4_u64 + CB_HEADER_U64; // "as" and math is safe because of early checks
            buf_reader.seek(SeekFrom::Start(pos))?;
            buf_reader.read_exact(&mut bytes_vector)?;
            Ok(bytes_vector)
        })
        // Zip in the column of the output array
        .zip(out_val.axis_iter_mut(nd::Axis(1)))
        // In parallel, decompress the iid info and put it in its column
        .try_for_each(|(bytes_vector_result, mut col)| match bytes_vector_result {
            Err(e) => Err(e),
            Ok(bytes_vector) => {
                for out_iid_i in 0..iid_index.len() {
                    let i_div_4: usize = i_div_4_array[out_iid_i];
                    let i_mod_4_times_2 = i_mod_4_times_2_array[out_iid_i];
                    let genotype_byte: u8 = (bytes_vector[i_div_4] >> i_mod_4_times_2) & 0x03;
                    col[out_iid_i] = from_two_bits_to_value[genotype_byte as usize];
                }
                Ok(())
            }
        })?;

    Ok(())
}

#[anyinput]
fn path_ref_to_string(path: AnyPath) -> String {
    PathBuf::from(path).display().to_string()
}

#[anyinput]
pub fn open_and_check(path: AnyPath) -> Result<(BufReader<File>, Vec<u8>), BedErrorPlus> {
    let mut buf_reader = BufReader::new(File::open(path)?);
    let mut bytes_vector: Vec<u8> = vec![0; CB_HEADER_USIZE];
    buf_reader.read_exact(&mut bytes_vector)?;
    if (BED_FILE_MAGIC1 != bytes_vector[0]) || (BED_FILE_MAGIC2 != bytes_vector[1]) {
        return Err(BedError::IllFormed(path_ref_to_string(path)).into());
    }
    Ok((buf_reader, bytes_vector))
}

fn set_up_two_bits_to_value<TVal: From<i8>>(count_a1: bool, missing_value: TVal) -> [TVal; 4] {
    let homozygous_primary_allele = TVal::from(0); // Major Allele
    let heterozygous_allele = TVal::from(1);
    let homozygous_secondary_allele = TVal::from(2); // Minor Allele

    if count_a1 {
        [
            homozygous_secondary_allele, // look-up 0
            missing_value,               // look-up 1
            heterozygous_allele,         // look-up 2
            homozygous_primary_allele,   // look-up 3
        ]
    } else {
        [
            homozygous_primary_allele,   // look-up 0
            missing_value,               // look-up 1
            heterozygous_allele,         // look-up 2
            homozygous_secondary_allele, // look-up 3
        ]
    }
}

type Array1Usize = nd::ArrayBase<nd::OwnedRepr<usize>, nd::Dim<[usize; 1]>>;
type Array1U8 = nd::ArrayBase<nd::OwnedRepr<u8>, nd::Dim<[usize; 1]>>;

pub fn check_and_precompute_iid_index(
    in_iid_count: usize,
    iid_index: &[isize],
) -> Result<(Array1Usize, Array1U8), BedErrorPlus> {
    let lower_iid_count = -(in_iid_count as isize);
    let upper_iid_count: isize = (in_iid_count as isize) - 1;
    let mut i_div_4_array = nd::Array1::<usize>::zeros(iid_index.len());
    let mut i_mod_4_times_2_array = nd::Array1::<u8>::zeros(iid_index.len());
    let mut result_list: Vec<Result<(), BedError>> = vec![Ok(()); iid_index.len()];
    nd::par_azip!((in_iid_i_signed in iid_index,
        i_div_4 in &mut i_div_4_array,
        i_mod_4_times_2 in &mut i_mod_4_times_2_array,
        result in &mut result_list
    )
    {
        let in_iid_i = if (0..=upper_iid_count).contains(in_iid_i_signed) {
            *result = Ok(());
            *in_iid_i_signed as usize
        } else if (lower_iid_count..=-1).contains(in_iid_i_signed) {
            *result = Ok(());
            in_iid_count - ((-in_iid_i_signed) as usize)
        } else {
            *result = Err(BedError::IidIndexTooBig(
                *in_iid_i_signed,
            ));
            0
        };
        *i_div_4 = in_iid_i / 4;
        *i_mod_4_times_2 = (in_iid_i % 4 * 2) as u8;
    });
    result_list
        .iter()
        .par_bridge()
        .try_for_each(|x| (*x).clone())?;
    Ok((i_div_4_array, i_mod_4_times_2_array))
}

pub trait Max {
    fn max() -> Self;
}

impl Max for u8 {
    fn max() -> u8 {
        std::u8::MAX
    }
}

impl Max for u64 {
    fn max() -> u64 {
        std::u64::MAX
    }
}

pub fn try_div_4<T: Max + TryFrom<usize> + Sub<Output = T> + Div<Output = T> + Ord>(
    in_iid_count: usize,
    in_sid_count: usize,
    cb_header: T,
) -> Result<(usize, T), BedErrorPlus> {
    // 4 genotypes per byte so round up without overflow
    let in_iid_count_div4 = if in_iid_count > 0 {
        (in_iid_count - 1) / 4 + 1
    } else {
        0
    };
    let in_iid_count_div4_t = match T::try_from(in_iid_count_div4) {
        Ok(v) => v,
        Err(_) => return Err(BedError::IndexesTooBigForFiles(in_iid_count, in_sid_count).into()),
    };
    let in_sid_count_t = match T::try_from(in_sid_count) {
        Ok(v) => v,
        Err(_) => return Err(BedError::IndexesTooBigForFiles(in_iid_count, in_sid_count).into()),
    };

    let m: T = Max::max(); // Don't know how to move this into the next line.
    if in_sid_count > 0 && (m - cb_header) / in_sid_count_t < in_iid_count_div4_t {
        return Err(BedError::IndexesTooBigForFiles(in_iid_count, in_sid_count).into());
    }

    Ok((in_iid_count_div4, in_iid_count_div4_t))
}

/// A trait alias, used internally, to provide default missing values for i8,
/// f32, f64.
pub trait Missing {
    /// The default missing value for a type such as i8, f32, and f64.
    fn missing() -> Self;
}

impl Missing for f64 {
    fn missing() -> Self {
        f64::NAN
    }
}

impl Missing for f32 {
    fn missing() -> Self {
        f32::NAN
    }
}

impl Missing for i8 {
    fn missing() -> Self {
        -127i8
    }
}
