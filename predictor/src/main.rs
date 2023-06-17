extern crate blas_src;

mod join;
mod predict;

use anyhow::Result;
use join::match_snp;
use ndarray::s;
use polars::prelude::DataFrame;
use predict::{cal_scores_onethread, cal_scores_par};
use reader::{BedReaderNoLib, Beta};

fn read_data(beta_path: &str, bed_path: &str) -> Result<(Beta, BedReaderNoLib)> {
    let beta = Beta::new(beta_path)?;

    let bed = BedReaderNoLib::new(bed_path)?;
    return Ok((beta, bed));
}

fn main() {
    let beta_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/model_demo/Weights.tsv";
    let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG";
    let score_names = vec!["Lassosum".to_string()];
    let batch_size = 8000;
    let thread_num: usize = 1;

    let (beta, bed) = read_data(beta_path, bed_path).unwrap();
    let weights = match_snp(&bed.bim, &beta, &mut score_names.clone()).unwrap();

    let scores: DataFrame;
    if thread_num == 1 {
        scores = cal_scores_onethread(batch_size, &weights, &bed, &score_names).unwrap();
    } else {
        scores = cal_scores_par(5, batch_size, weights, bed, score_names).unwrap();
    }
    dbg!("{}", scores);
}
