use std::sync::Arc;

use anyhow::Result;
use polars::prelude::{CsvReader, DataFrame, DataType, Field, Schema, SerReader};

pub fn read_fam(fam_path: &str) -> Result<DataFrame> {
    let my_schmema = get_fam_schema();
    let mut fam = CsvReader::from_path(fam_path)?
        .with_delimiter(b'\t')
        .with_schema(Arc::new(my_schmema))
        .has_header(false)
        .finish()?;
    fam = fam.select(["FID", "IID", "SEX", "PHENO"])?;
    fam = fam.with_row_count("IDX", None)?;
    Ok(fam)
}

pub fn read_bim(bim_path: &str) -> Result<DataFrame> {
    let my_schmema = get_bim_schema();
    let mut bim = CsvReader::from_path(bim_path)?
        .with_delimiter(b'\t')
        .with_schema(Arc::new(my_schmema))
        .has_header(false)
        .finish()?;
    bim = bim.select(["CHR", "ID", "POS", "REF", "ALT"])?;
    bim = bim.with_row_count("IDX", None)?;
    Ok(bim)
}

fn get_fam_schema() -> Schema {
    Schema::from_iter(
        vec![
            Field::new("FID", DataType::Utf8),
            Field::new("IID", DataType::Utf8),
            Field::new("P", DataType::Utf8),
            Field::new("M", DataType::Utf8),
            Field::new("SEX", DataType::Int32),
            Field::new("PHENO", DataType::Float32),
        ]
        .into_iter(),
    )
}

fn get_bim_schema() -> Schema {
    Schema::from_iter(
        vec![
            Field::new("CHR", DataType::Utf8),
            Field::new("ID", DataType::Utf8),
            Field::new("cM", DataType::Int32),
            Field::new("POS", DataType::Int32),
            Field::new("ALT", DataType::Utf8),
            Field::new("REF", DataType::Utf8),
        ]
        .into_iter(),
    )
}
