mod arg;
mod read_beta;
use std::{fs::File, sync::Arc};

use anyhow::Result;
pub use arg::Args;
use polars::prelude::{CsvReader, CsvWriter, DataFrame, SerReader, SerWriter};
use read_beta::get_beta_schema;
use reader::BedReaderNoLib;

pub fn read_data(cli: &Args) -> Result<(DataFrame, Vec<String>, BedReaderNoLib)> {
    let (my_schmema, cols) = get_beta_schema(cli)?;
    let beta = CsvReader::from_path(&cli.weight_path)?
        .with_delimiter(b'\t')
        .with_schema(Arc::new(my_schmema))
        .has_header(true)
        .finish()?;

    let bed = BedReaderNoLib::new(&cli.bed_path)?;
    return Ok((beta, cols, bed));
}

pub fn write_file(out_path: &str, scores: &mut DataFrame) -> Result<()> {
    let out_path: File = File::create(out_path).unwrap();
    CsvWriter::new(out_path).has_header(true).finish(scores)?;
    Ok(())
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
