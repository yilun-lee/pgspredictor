//pub mod read_bed;
pub mod read_bed_nolib;
mod test;

use anyhow::Result;
use ndarray::Array2;
use polars::{
    frame::DataFrame,
    prelude::{DataType, Field, Schema},
};

pub trait ReadGenotype {
    type GenoDtype;
    type GenoIdx;
    fn get_geno(&self, sid: &Self::GenoIdx, iid: &Self::GenoIdx)
        -> Result<Array2<Self::GenoDtype>>;

    fn get_ind(&self, iid: &Self::GenoIdx, inv: bool) -> Result<DataFrame>;
    fn get_snp(&self, sid: &Self::GenoIdx, inv: bool) -> Result<DataFrame>;

    fn get_ind_schema() -> Schema {
        Schema::from_iter(
            vec![
                Field::new("IID", DataType::Utf8),
                Field::new("PHENO", DataType::Float32),
                Field::new("SEX", DataType::Int8),
            ]
            .into_iter(),
        )
    }

    fn get_snp_schema() -> Schema {
        Schema::from_iter(
            vec![
                Field::new("ID", DataType::Utf8),
                Field::new("CHR", DataType::Utf8),
                Field::new("POS", DataType::UInt8),
                Field::new("REF", DataType::Utf8),
                Field::new("ALT", DataType::Utf8),
            ]
            .into_iter(),
        )
    }
}
