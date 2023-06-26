use log::{info, LevelFilter};

use crate::args::MyArgs;

pub fn print_run_config(cli: &MyArgs) {
    let aa: &str;
    if !cli.batch_ind {
        aa = "snp";
    } else {
        aa = "ind";
    }
    info!(
        "Run with batch size {} and {} threads along {}",
        cli.batch_size, cli.thread_num, aa
    );
}

pub fn match_verbose(verbose_count: u8) -> LevelFilter {
    let my_level = match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    my_level
}
