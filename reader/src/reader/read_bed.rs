use anyhow::Result;
use bed_reader::{Bed, ReadOptions};
use ndarray::Array2;
use polars::prelude::*;

use super::ReadGenotype;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BedReader {
    pub bed_path: String,
    pub bed: Bed,
    pub fam: DataFrame,
    pub bim: DataFrame,
}

#[allow(dead_code)]
fn read_bim(bed: &mut Bed) -> Result<DataFrame> {
    let id = Series::new("ID", bed.sid()?.to_vec());
    let chrom = Series::new("CHR", bed.chromosome()?.to_vec());
    let pos = Series::new("POS", bed.bp_position()?.to_vec());
    let a0: Series = Series::new("ALT", bed.allele_1()?.to_vec());
    let a1: Series = Series::new("REF", bed.allele_2()?.to_vec());

    let mut bim = DataFrame::new(vec![id, chrom, pos, a0, a1])?;
    bim = bim.with_row_count("IDX", None)?;

    Ok(bim)
}

#[allow(dead_code)]
fn read_fam(bed: &mut Bed) -> Result<DataFrame> {
    let iid = Series::new("IID", bed.iid()?.to_vec());
    let pheno = Series::new("PHENO", bed.pheno()?.to_vec());
    let sex: Series = Series::new("SEX", bed.sex()?.to_vec());

    let mut fam = DataFrame::new(vec![iid, pheno, sex])?;
    fam = fam.with_row_count("IDX", None)?;
    Ok(fam)
}

impl BedReader {
    pub fn new(bed_path: &str) -> Result<BedReader> {
        let mut bed = Bed::builder(bed_path).build()?;
        let fam = read_fam(&mut bed)?;
        let bim = read_bim(&mut bed)?;
        Ok(BedReader {
            bed_path: bed_path.to_string(),
            bed: bed,
            fam: fam,
            bim: bim,
        })
    }
}

impl BedReader {
    fn create_mask(
        &self,
        v: &Vec<isize>,
        inv: bool,
        data_frame: &DataFrame,
    ) -> Result<ChunkedArray<BooleanType>> {
        let v: Vec<u32> = v.iter().map(|&e| e as u32).collect();
        let v: Series = Series::new("whatever", v);
        let mask = data_frame.column("IDX")?;
        let mask = match inv {
            true => mask.is_in(&v)?,
            false => !mask.is_in(&v)?,
        };

        Ok(mask)
    }
}

impl ReadGenotype for BedReader {
    type GenoDtype = f32;
    type GenoIdx = Option<Vec<isize>>;

    fn get_geno(self, sid: &Self::GenoIdx, iid: &Self::GenoIdx) -> Result<Array2<Self::GenoDtype>> {
        let mut builder = ReadOptions::builder();
        let mut builder = match iid {
            Some(v) => builder.iid_index(v),
            None => &mut builder,
        };

        let builder = match sid {
            Some(v) => builder.sid_index(v),
            None => &mut builder,
        };
        // dont know how to fix
        let val = builder.f32().read(self.bed)?;
        Ok(val)
    }

    // TODO is this DataFrame borrow ?
    fn get_ind(&mut self, iid: &Self::GenoIdx, inv: bool) -> Result<DataFrame> {
        if let Some(v) = iid {
            let mask = self.create_mask(v, inv, &self.fam)?;
            let aa = self.fam.filter(&mask)?;
            return Ok(aa);
        }
        return Ok(self.fam.clone());
    }

    fn get_snp(&mut self, iid: &Self::GenoIdx, inv: bool) -> Result<DataFrame> {
        if let Some(v) = iid {
            let mask = self.create_mask(v, inv, &self.bim)?;
            let aa = self.bim.filter(&mask)?;
            return Ok(aa);
        }
        return Ok(self.bim.clone());
    }
}
