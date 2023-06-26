mod bed_crate;
mod bed_error;
mod read_meta;
use std::{path::Path, thread::available_parallelism};

use anyhow::{anyhow, Result};
use bed_crate::read_no_alloc;
use ndarray::{Array2, ShapeBuilder};
use polars::{
    prelude::{BooleanType, ChunkedArray, DataFrame, NamedFrom},
    series::Series,
};
use read_meta::{read_bim, read_fam};

// codes are copy from https://github.com/fastlmm/bed-reader/blob/master/src/lib.rs
use super::ReadGenotype;

#[derive(Clone, Debug)]
pub struct BedReaderNoLib {
    pub bed_path: String,
    pub fam: DataFrame,
    pub bim: DataFrame,
    pub iid_count: usize,
    pub sid_count: usize,
    pub iid_idx: Vec<isize>,
    pub sid_idx: Vec<isize>,
}

impl BedReaderNoLib {
    pub fn new(bfile_path: &str) -> Result<BedReaderNoLib> {
        // get path
        let bed_path = format!("{}.bed", bfile_path);
        let fam_path = format!("{}.fam", bfile_path);
        let bim_path = format!("{}.bim", bfile_path);
        if !Path::new(&bed_path).exists() {
            return Err(anyhow!("path {} not exists", bfile_path));
        }
        // get fam, bim
        let fam = read_fam(&fam_path)?;
        let bim = read_bim(&bim_path)?;

        let iid_count = fam.shape().0;
        let sid_count = bim.shape().0;

        let aa = iid_count as isize;
        let iid_all: Vec<isize> = (0..aa).collect();

        let aa = sid_count as isize;
        let sid_all: Vec<isize> = (0..aa).collect();

        Ok(BedReaderNoLib {
            bed_path,
            fam,
            bim,
            iid_count,
            sid_count,
            iid_idx: iid_all,
            sid_idx: sid_all,
        })
    }
}

impl ReadGenotype for BedReaderNoLib {
    type GenoDtype = f32;
    type GenoIdx = Option<Vec<isize>>;
    fn get_geno(
        &self,
        sid: &Self::GenoIdx,
        iid: &Self::GenoIdx,
    ) -> Result<Array2<Self::GenoDtype>> {
        let iid = match iid {
            Some(v) => v,
            None => &self.iid_idx,
        };
        let sid = match sid {
            Some(v) => v,
            None => &self.sid_idx,
        };
        let shape = ShapeBuilder::set_f((iid.len(), sid.len()), false);
        let mut val = Array2::<f32>::default(shape);

        read_no_alloc(
            &self.bed_path,
            self.iid_count,
            self.sid_count,
            true,
            iid,
            sid,
            f32::NAN,
            available_parallelism()?.get(),
            &mut val.view_mut(),
        )?;
        Ok(val)
    }

    fn get_ind(&self, iid: &Self::GenoIdx, inv: bool) -> Result<DataFrame> {
        if let Some(v) = iid {
            let mask: ChunkedArray<BooleanType> = create_mask(v, inv, &self.fam)?;
            let aa = self.fam.filter(&mask)?;
            return Ok(aa);
        }
        Ok(self.fam.clone())
    }
    fn get_snp(&self, sid: &Self::GenoIdx, inv: bool) -> Result<DataFrame> {
        if let Some(v) = sid {
            let mask = create_mask(v, inv, &self.bim)?;
            let aa = self.bim.filter(&mask)?;
            return Ok(aa);
        }
        Ok(self.bim.clone())
    }
}

fn create_mask(
    v: &[isize],
    inv: bool,
    data_frame: &DataFrame,
) -> Result<ChunkedArray<BooleanType>> {
    let v: Vec<u32> = v.iter().map(|&e| e as u32).collect();
    let v: Series = Series::new("whatever", v);
    let mask = data_frame.column("IDX")?;
    let mask = match inv {
        false => mask.is_in(&v)?,
        true => !mask.is_in(&v)?,
    };

    Ok(mask)
}
