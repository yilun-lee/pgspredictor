extern crate blas_src;

mod args;
mod join;
mod predict;
mod runner;

use args::Args;
use clap::Parser;
use genoreader::BedReaderNoLib;

use crate::{
    args::MissingStrategy,
    runner::{save_as_json, write_file, Runner},
};

fn main() {
    let cli = Args::parse();

    let runner = Runner::from_args(&cli).unwrap();

    // read
    let bed = BedReaderNoLib::new(&cli.bed_path).unwrap();
    let (mut scores, match_status) = runner.run_batch_ind(bed).unwrap();

    // write
    let out_path = format!("{}.check.json", &cli.out_path);
    save_as_json(match_status, &out_path).unwrap();
    let out_path = format!("{}.score.csv", &cli.out_path);
    write_file(&out_path, &mut scores).unwrap();

    println!("{}", scores);
}
