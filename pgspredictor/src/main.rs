#![doc = include_str!("../../README.md")]

mod args;
mod runner;
mod utils;


use args::MyArgs;
use clap::Parser;
use env_logger::Builder;
use genoreader::{BedReaderNoLib, BfileSet};
use log::{debug, info};
use polars::{export::chrono, prelude::DataFrame};
use predictor::join::MatchStatus;

use crate::{
    runner::{post::PgsResult, Runner},
    utils::{match_verbose, print_run_config},
};

fn main_fn() {
    // parse
    let cli = MyArgs::parse();
    // get logger
    let my_level = match_verbose(cli.verbose);
    Builder::new().filter_level(my_level).init();
    let time_stamp = chrono::Utc::now();
    info!("Start pgs-predictor on {time_stamp}");

    // parse to Runner obj
    let runner = Runner::from_args(&cli).unwrap();
    print_run_config(&cli);

    // batch by snp or ind
    let mut scores: DataFrame;
    let match_status: MatchStatus;
    if !cli.batch_ind {
        let bfileset = BfileSet::new(&cli.bed_path).unwrap();
        debug!(
            "Successfully load bfile with {} snp and {} ind",
            &bfileset.bim.height(), &bfileset.fam.height()
        );
        (scores, match_status) = runner.run_batch_snp(bfileset).unwrap();
    } else {
        let bed = BedReaderNoLib::new(&cli.bed_path).unwrap();
        debug!(
            "Successfully load bfile with {} snp and {} ind",
            &bed.sid_count, &bed.iid_count
        );
        (scores, match_status) = runner.run_batch_ind(bed).unwrap();
    }
    info!(
        "There are {} snps matched between bfile ({} snp) and beta ({} snp)",
        match_status.match_snp, match_status.bfile_snp, match_status.model_snp
    );
    debug!("{}", scores);

    // write
    let mut pgs_score = PgsResult::new(
        &mut scores,
        match_status,
        &cli.score_names,
        &cli.out_prefix,
        cli.percentile_flag,
        &cli.rank_path,
        cli.eval_flag,
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
    use std::fs::File;
    if let Ok(report) = guard.report().build() {
        let file = File::create("flamegraph.svg").unwrap();
        let mut options = pprof::flamegraph::Options::default();
        options.image_width = Some(2500);
        report.flamegraph_with_options(file, &mut options).unwrap();
    };
    */
    
}
