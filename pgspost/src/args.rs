use anyhow::{Result, anyhow};
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};
use polars::export::chrono;

/// Command line argument
#[derive(Parser, Debug)]
#[command(
    name = "pgspredictor.rs", 
    author = "Yilun.lee", 
    version = "0.1.0", 
    about = "A pgs predictor written in rust", long_about = None)]
#[command(next_line_help = true)]
#[command(propagate_version = true)]
pub struct MyArgs {
    /// mode, should be Predict / Validate 
    pub mode: String,

    /// weight path, should be a tsv file
    pub score_path: String,

    /// output prefix
    #[arg(short, long)]
    pub out_prefix: String,

    /// score names: scores to be process
    #[arg(short = 'n', long)]
    pub score_names: Vec<String>,

    /// whether to show log, use -v -vv -vvv to present increase log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// whether to calculate correlation between PHENO and score
    #[arg(short = 'E', long, default_value_t = false)]
    pub eval_flag: bool,
    
    /// path to rank file produce by pgs-predictor. RANK as the first column,
    /// which is 0~100, and the other column are score names. If specified,
    /// percentiles of sample scores are interpolated based on the rank.
    #[arg(short = 'R', long)]
    pub rank_path: Option<String>,

    /// covariate path. a csv containing covariate for each sample 
    /// FID IID columns should be present. Other columns will be regarded as covariate and intepreted as f32.
    #[arg(short = 'C', long,)]
    pub cov_csv_path: Option<String>,

    /// covariate path. a json containing covariate weights, used in predict mode
    /// when cov_weight_path is specified, cov_csv_path is required to be presented
    #[arg(short = 'c', long,)]
    pub cov_weight_path: Option<String>,
}

#[derive(Debug)]
enum ModeEnum{
    Predict,
    Validate,
    Run,
}

impl ModeEnum {
    fn from_str(ss: &str) -> Result<ModeEnum>{
        let ss = ss.to_ascii_lowercase();
        if ss == "validate" || ss == "val" {
            Ok(ModeEnum::Validate)
        } else if ss == "predict" || ss == "pred" {
            Ok(ModeEnum::Predict)
        } else if ss == "run"  {
            Ok(ModeEnum::Run)
        } else {
            Err(anyhow!("Mode not found, should be one of the following: Validate, Predict, Test, Run, CandT, got {}", ss))
        }
    }
}

impl MyArgs {
    // ["Validate", "Predict", "Test", "Run", "CandT"]
    pub fn check_defaul(&mut self) -> Result<()>{
        let mode = ModeEnum::from_str(&self.mode)?;
        info!("Got mode {:?}, check arg accordingly", mode);
        match mode {
            ModeEnum::Validate => {
                assert!(self.rank_path.is_none(), "--rank-path (-R) should be None in Validate mode");
                assert!(self.cov_weight_path.is_none(), "--cov-weight-path (-c) should be None in Validate mode");
                self.eval_flag = true;
            },
            ModeEnum::Predict => {
                assert!(self.rank_path.is_some(), "--rank-path (-R) should be specified in Predict mode");
                if self.cov_csv_path.is_none() != self.cov_weight_path.is_none() {
                    return Err(anyhow!("--cov-csv-path (-C) and --cov-weight-path (-c) should be both present or either not present in Predict mode"))
                }
                self.eval_flag = false;
            },
            ModeEnum::Run => (),
        }
        Ok(())
    }

}

pub fn match_log(verbose: u8){
    let my_level = match_verbose(verbose);
    Builder::new().filter_level(my_level).init();
    let time_stamp = chrono::Utc::now();
    info!("Start pgs-predictor on {time_stamp}");
}

fn match_verbose(verbose_count: u8) -> LevelFilter {
    match verbose_count {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}
