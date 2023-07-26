use env_logger::Builder;
use log::{info, LevelFilter};
use polars::export::chrono;

use crate::args::MyArgs;

pub fn print_run_config(cli: &MyArgs) {
    let aa = if !cli.batch_ind { "snp" } else { "ind" };
    info!(
        "Run with batch size {} and {} threads along {}",
        cli.batch_size, cli.thread_num, aa
    );
}

fn match_verbose(verbose_count: u8) -> LevelFilter {
    match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

pub fn match_log(verbose: u8){
    let my_level = match_verbose(verbose);
    Builder::new().filter_level(my_level).init();
    let time_stamp = chrono::Utc::now();
    info!("Start pgs-predictor on {time_stamp}");
}

