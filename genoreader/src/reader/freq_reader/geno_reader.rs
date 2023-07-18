use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use anyhow::{anyhow, Result};
use super::bit_op::{nonmissing_mask_u8,set_up_two_bits_to_value};
use ndarray::{s, Array, Ix2, ArrayBase, ViewRepr, Dim, Ix1};

use crate::reader::read_bed_nolib::bed_crate::{open_and_check, try_div_4, check_and_precompute_iid_index};

#[allow(dead_code)]
const BED_FILE_MAGIC1: u8 = 0x6C; // 0b01101100 or 'l' (lowercase 'L')
#[allow(dead_code)]
const BED_FILE_MAGIC2: u8 = 0x1B; // 0b00011011 or <esc>
const CB_HEADER_U64: u64 = 3;
#[allow(dead_code)]
const CB_HEADER_USIZE: usize = 3;

#[allow(dead_code)]
pub struct BedSnpReader {
    pub reader: BufReader<File>,
    in_iid_count_div4_u64: u64,
    in_iid_count_div4: usize,
    in_iid_count: usize,
    in_sid_count: usize,
    iid_idx: Vec<usize>,
    bit_map: [f32; 4],
}

impl BedSnpReader {
    pub fn new(
        bed_path: impl AsRef<Path>,
        in_iid_count: usize,
        in_sid_count: usize,
    ) -> Result<BedSnpReader> {
        let (mut buf_reader, bytes_vector) = open_and_check(bed_path)?;
        let (in_iid_count, in_sid_count) = match bytes_vector[2] {
            0 => (in_sid_count, in_iid_count),
            1 => (in_iid_count, in_sid_count),
            _ => {
                return Err(anyhow!(
                    "bed file may be malformed, the 3th magic number should not be other than 0 1"
                ))
            }
        };
        let (in_iid_count_div4, in_iid_count_div4_u64) =
            try_div_4(in_iid_count, in_sid_count, CB_HEADER_U64)?;
        // "as" and math is safe because of early checks
        let file_len = buf_reader.seek(SeekFrom::End(0))?;

        let file_len2: u64 = in_iid_count_div4_u64 * (in_sid_count as u64) + CB_HEADER_U64;
        if file_len != file_len2 {
            return Err(anyhow!("bed file may be malformed, size is not reasonable"));
        }
        let bit_map = set_up_two_bits_to_value(true, 0.);
        // some predefine things
        let iid_idx: Vec<usize> = (0..in_iid_count_div4).map(|x| x * 4).collect();

        // set up bit_map
        let bed_snp_reder = BedSnpReader {
            reader: buf_reader,
            in_iid_count_div4_u64,
            in_iid_count_div4,
            in_iid_count,
            in_sid_count,
            iid_idx,
            bit_map,
        };
        Ok(bed_snp_reder)
    }

    fn read_snp(&mut self, sid_idx: u64) -> Result<Vec<u8>> {
        let mut bytes_vector: Vec<u8> = vec![0; self.in_iid_count_div4];
        let pos: u64 = sid_idx * self.in_iid_count_div4_u64 + CB_HEADER_U64; // "as" and math is safe because of early checks
        self.reader.seek(SeekFrom::Start(pos))?;
        self.reader.read_exact(&mut bytes_vector)?;
        Ok(bytes_vector)
    }

    fn truncate_geno(&self, mut val: Array::<f32, Ix2>) -> Array::<f32, Ix2>{
        if self.in_iid_count % 4 != 0 {
            val = val.slice(s![..self.in_iid_count, ..]).to_owned();
        }
        val
    }

    pub fn read_to_ndarray(
        &mut self,
        sid_idxs: &[isize],
        swap_vec: &[bool],
    ) -> Result<(Array<f32, Ix2>,Vec<f32>)> {
        let total_iid = self.in_iid_count_div4 * 4;
        let mut val = Array::<f32, Ix2>::default((total_iid, sid_idxs.len()));
        let mut freq_vec: Vec<f32> = vec![];
        // read by each snp
        sid_idxs
            .iter()
            .zip(swap_vec.iter())
            .zip(val.axis_iter_mut(ndarray::Axis(1)))
            .try_for_each(|((idx, swap_flag), col)| -> Result<()> {
                // read
                let byte_vec: Vec<u8> = self.read_snp(*idx as u64)?;
                // calculate freq
                let freq = byte_vec_to_freq(&byte_vec);
                self.bit_map[1] = freq;
                freq_vec.push(freq);
                // into array
                byte_vec_to_arr(byte_vec, *swap_flag, &self.iid_idx, col, &self.bit_map);
                Ok(())
            })?;

        // truncate extra 0
        val = self.truncate_geno(val);
        Ok((val,freq_vec))
    }

    pub fn read_to_ndarray_freq(
        &mut self,
        sid_idxs: &[isize],
        swap_vec: &[bool],
        freq_vec: &[f32],
    ) -> Result<Array<f32, Ix2>> {
        let total_iid = self.in_iid_count_div4 * 4;
        let mut val = Array::<f32, Ix2>::default((total_iid, sid_idxs.len()));

        // read by each snp
        sid_idxs
            .iter()
            .zip(swap_vec.iter())
            .zip(freq_vec.iter())
            .zip(val.axis_iter_mut(ndarray::Axis(1)))
            .try_for_each(|(((idx, swap_flag), freq), col)| -> Result<()> {
                // read
                let byte_vec: Vec<u8> = self.read_snp(*idx as u64)?;
                // calculate freq
                self.bit_map[1] = *freq;
                // into array
                byte_vec_to_arr(byte_vec, *swap_flag, &self.iid_idx, col, &self.bit_map);
                Ok(())
            })?;

        // truncate extra 0
        val = self.truncate_geno(val);
        Ok(val)
    }

    pub fn read_to_ndarray_ind(
        &mut self,
        sid_idxs: &[isize],
        iid_idxs: &[isize],
        swap_vec: &[bool],
    ) -> Result<Array<f32, Ix2>> {
        // Check the file length
        let mut val = Array::<f32, Ix2>::default((iid_idxs.len(), sid_idxs.len()));

        // Check and precompute for each iid_index
        let (i_div_4_array, i_mod_4_times_2_array) =
            check_and_precompute_iid_index(self.in_iid_count, iid_idxs)?;
    
        // Possible optimization: We could try to read only the iid info needed
        // Possible optimization: We could read snp in their input order instead of
        // their output order
        sid_idxs
            .iter()
            // Zip in the column of the output array
            .zip(swap_vec.iter())
            .zip(val.axis_iter_mut(ndarray::Axis(1)))
            // In parallel, decompress the iid info and put it in its column
            .try_for_each(|((idx, swap_flag), mut col)| -> Result<()> {
                let byte_vec: Vec<u8> = self.read_snp(*idx as u64)?;
                for out_iid_i in 0..iid_idxs.len() {
                    let i_div_4: usize = i_div_4_array[out_iid_i];
                    let i_mod_4_times_2 = i_mod_4_times_2_array[out_iid_i];
                    let genotype_byte: u8 = (byte_vec[i_div_4] >> i_mod_4_times_2) & 0x03;
                    if *swap_flag {
                        col[out_iid_i] = 2.-self.bit_map[genotype_byte as usize];
                    } else {
                        col[out_iid_i] = self.bit_map[genotype_byte as usize];
                    }
                }
                Ok(())
            })?;
    
        Ok(val)
    }
    
}


fn byte_vec_to_freq(byte_vec: &[u8]) -> f32 {
    let (nonmissing_count, ones_count) = byte_vec.iter().fold(
        (0, 0),
        |(mut nonmissing_count, mut ones_count), byte| -> (u32, u32) {
            let nonmissing_mask = nonmissing_mask_u8(*byte);
            nonmissing_count += nonmissing_mask.count_ones();
            ones_count += (*byte & nonmissing_mask).count_ones();
            (nonmissing_count, ones_count)
        },
    );
    let freq: f32 = (1. - (ones_count as f32) / (nonmissing_count as f32)) * 2.;
    freq
}


fn byte_vec_to_arr(byte_vec: Vec<u8>, 
    swap_flag: bool, iid_idx: &[usize], mut col: ArrayBase<ViewRepr<&mut f32>, Dim<[usize; 1]>>, 
    bit_map: &[f32]){
    if !swap_flag{
        // into array
        byte_vec
            .into_iter()
            .zip(iid_idx.iter())
            .for_each(|(byte, idx)| {
                col[*idx] = bit_map[(byte & 3) as usize];
                col[idx + 1] = bit_map[((byte >> 2) & 3) as usize];
                col[idx + 2] = bit_map[((byte >> 4) & 3) as usize];
                col[idx + 3] = bit_map[((byte >> 6) & 3) as usize];
            });
        }else{
            byte_vec
            .into_iter()
            .zip(iid_idx.iter())
            .for_each(|(byte, idx)| {
                col[*idx] = 2. - bit_map[(byte & 3) as usize];
                col[idx + 1] = 2. - bit_map[((byte >> 2) & 3) as usize];
                col[idx + 2] = 2. - bit_map[((byte >> 4) & 3) as usize];
                col[idx + 3] = 2. - bit_map[((byte >> 6) & 3) as usize];
            });
        }
}
