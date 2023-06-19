extern crate blas_src;

mod join;
mod predict;
mod tools;
use clap::Parser;
use join::match_snp;
use polars::prelude::DataFrame;
use predict::{cal_scores_onethread, cal_scores_par};
use tools::{read_data, save_as_json, write_file, Args};

fn main() {
    let cli = Args::parse();
    let score_names = &cli.score_names;

    // load and match
    let (beta, bed) = read_data(&cli).unwrap();
    let (weights, match_status) = match_snp(
        &bed.bim,
        &beta,
        &mut &score_names.clone(),
        cli.freq_flag,
        cli.match_id_flag,
    )
    .unwrap();

    // run
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

    // write
    let out_path = format!("{}.check.json", &cli.out_path);
    save_as_json(match_status, &out_path).unwrap();
    let out_path = format!("{}.score.csv", &cli.out_path);
    write_file(&out_path, &mut scores).unwrap();

    println!("{}", scores);
}
