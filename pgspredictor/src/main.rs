#![doc = include_str!("../../README.md")]

mod args;
mod runner;

use args::Args;
use clap::Parser;
use env_logger::Builder;
use genoreader::BedReaderNoLib;
use log::{debug, info, LevelFilter};
use polars::{export::chrono, prelude::DataFrame};
use predictor::join::MatchStatus;

use crate::runner::{post::PgsResult, Runner};

fn print_run_config(cli: &Args) {
    let aa: &str;
    if cli.batch_snp {
        aa = "snp";
    } else {
        aa = "ind";
    }
    info!(
        "Run with batch size {} and {} threads along {}",
        cli.batch_size, cli.thread_num, aa
    );
}

fn main_fn() {
    // parse
    let cli = Args::parse();
    // get logger
    if !cli.verbose {
        Builder::new().filter_level(LevelFilter::Warn).init();
    } else {
        Builder::new().filter_level(LevelFilter::Debug).init();
    }
    let time_stamp = chrono::Utc::now();
    info!("Start pgs-predictor on {time_stamp}");

    // read bed
    let bed = BedReaderNoLib::new(&cli.bed_path).unwrap();
    info!(
        "Successfully load bfile with {} snp and {} ind",
        &bed.sid_count, &bed.iid_count
    );

    // parse to Runner obj
    let runner = Runner::from_args(&cli).unwrap();
    print_run_config(&cli);
    // batch by snp or ind
    let mut scores: DataFrame;
    let match_status: MatchStatus;
    if cli.batch_snp {
        (scores, match_status) = runner.run_batch_snp(bed).unwrap();
    } else {
        (scores, match_status) = runner.run_batch_ind(bed).unwrap();
    }
    debug!("{}", scores);

    // write
    let mut pgs_score = PgsResult::new(
        &mut scores,
        match_status,
        &cli.score_names,
        &cli.out_prefix,
        cli.percentile_flag,
        &cli.rank_path,
    );
    pgs_score.write_output().unwrap();
    info!("Complete pgs-predictor!");
}

fn main() {
    /*
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();
    */
    main_fn();

    /*
    if let Ok(report) = guard.report().build() {
        let file = File::create("flamegraph.svg").unwrap();
        let mut options = pprof::flamegraph::Options::default();
        options.image_width = Some(2500);
        report.flamegraph_with_options(file, &mut options).unwrap();
    };
    */
}
