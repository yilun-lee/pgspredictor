extern crate blas_src;

mod args;
mod join;
mod predict;
mod runner;

use args::Args;
use clap::Parser;
use genoreader::BedReaderNoLib;
use polars::prelude::DataFrame;

use crate::{
    args::MissingStrategy,
    join::MatchStatus,
    runner::{save_as_json, write_file, Runner},
};

fn main() {
    let cli = Args::parse();

    // read bed
    let bed = BedReaderNoLib::new(&cli.bed_path).unwrap();

    // parse to Runner obj
    let runner = Runner::from_args(&cli).unwrap();
    // batch by snp or ind
    let mut scores: DataFrame;
    let match_status: MatchStatus;
    if cli.batch_snp {
        (scores, match_status) = runner.run_batch_snp(bed).unwrap();
    } else {
        (scores, match_status) = runner.run_batch_ind(bed).unwrap();
    }

    // write
    let out_path = format!("{}.check.json", &cli.out_path);
    save_as_json(serde_json::to_value(&match_status).unwrap(), &out_path).unwrap();
    let out_path = format!("{}.score.csv", &cli.out_path);
    write_file(&out_path, &mut scores).unwrap();

    println!("{}", scores);
}
