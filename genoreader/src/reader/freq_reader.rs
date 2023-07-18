mod bit_op;
mod geno_reader;
mod utils;

use std::{path::Path, sync::Arc};

use anyhow::{anyhow, Result};


use ndarray::{Array2, Array1};
use polars::prelude::{DataFrame, ChunkedArray, BooleanType};

use super::read_bed_nolib::read_meta::{read_bim, read_fam};
pub use geno_reader::BedSnpReader;
use utils::create_mask_u32;

pub struct BfileSet {
    pub bed_path: String,
    pub fam: DataFrame,
    pub bim: DataFrame,
}


impl BfileSet {
    pub fn new(bfile_path: &str) -> Result<BfileSet> {
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

        Ok(BfileSet {
            bed_path,
            fam,
            bim,
        })
    }

    pub fn get_ind(&self, iid: Option<&[u32]>, inv: bool) -> Result<DataFrame> {
        if let Some(v) = iid {
            let mask: ChunkedArray<BooleanType> = create_mask_u32(v, inv, &self.fam)?;
            let aa = self.fam.filter(&mask)?;
            return Ok(aa);
        }
        Ok(self.fam.clone())
    }

    pub fn get_snp(&self, sid: Option<&[u32]>, inv: bool) -> Result<DataFrame> {
        if let Some(v) = sid {
            let mask: ChunkedArray<BooleanType> = create_mask_u32(v, inv, &self.bim)?;
            let aa = self.bim.filter(&mask)?;
            return Ok(aa);
        }
        Ok(self.bim.clone())
    }
}



pub struct FreqBedReader {
    pub bed_reader: BedSnpReader,
    pub bfile_set: Arc<BfileSet>,
}

impl FreqBedReader {
    pub fn new(bfile_set: Arc<BfileSet>) -> Result<FreqBedReader> {

        let bed_reader: BedSnpReader = BedSnpReader::new(    
            &bfile_set.bed_path, bfile_set.fam.height(), bfile_set.bim.height()
        )?;

        Ok(FreqBedReader {
            bed_reader,
            bfile_set,
        })
    }

    pub fn read_snp(&mut self, snp_idx: &[isize], swap_vec: Option<&[bool]>, freq_vec: Option<&[f32]>) -> Result<(Array2<f32>, Option<Vec<f32>>)>{
        let default_swap_vec: Vec<bool> = vec![false; snp_idx.len()];
        let swap_vec = match swap_vec{
            Some(v) => v, 
            None => &default_swap_vec,
        };

        if freq_vec.is_none() {
            let (val, freq_vec) = self.bed_reader.read_to_ndarray(snp_idx, swap_vec)?;
            return Ok((val, Some(freq_vec)));
        } else {
            let val = self.bed_reader.read_to_ndarray_freq(snp_idx, swap_vec, freq_vec.unwrap())?;
            return Ok((val, None));
        };
    }


}





