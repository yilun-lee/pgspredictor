extern crate blas_src;

mod join;
mod predict;
mod tools;
use clap::Parser;
use join::match_snp;
use polars::prelude::DataFrame;
use predict::{cal_scores_onethread, cal_scores_par};
use tools::{read_data, write_file, Args};

fn main() {
    let cli = Args::parse();

    let score_names = &cli.score_names;

    let (beta, cols, bed) = read_data(&cli).unwrap();
    let weights = match_snp(
        &bed.bim,
        &beta,
        &mut &score_names.clone(),
        cli.freq_flag,
        &cols,
    )
    .unwrap();

    let mut scores: DataFrame;
    if cli.thread_num == 1 {
        scores = cal_scores_onethread(cli.batch_size, weights, &bed, score_names).unwrap();
    } else {
        scores = cal_scores_par(
            cli.thread_num,
            cli.batch_size,
            weights,
            bed,
            score_names.to_vec(),
        )
        .unwrap();
    }

    let out_path = format!("{}.score.csv", &cli.out_path);
    write_file(&out_path, &mut scores).unwrap();

    println!("{}", scores);
}
