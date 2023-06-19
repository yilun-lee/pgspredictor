use anyhow::{anyhow, Result};
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    name = "pgspredictor.rs", 
    author = "Yilun.lee", 
    version = "0.1.0", 
    about = "A pgs predictor written in rust", long_about = None)]
#[command(next_line_help = true)]
pub struct Args {
    /// weight path, should be a tsv file
    #[arg(short = 'm', long)]
    pub weight_path: String,

    /// path to plink bed files
    #[arg(short, long)]
    pub bed_path: String,

    /// output prefix
    #[arg(short, long)]
    pub out_path: String,

    /// score names: scores to be process
    #[arg(short = 'n', long)]
    pub score_names: Vec<String>,

    /// number of thread to run
    #[arg(short = 'T', long, default_value_t = 1)]
    pub thread_num: usize,

    /// batch size for sample
    #[arg(short = 'B', long, default_value_t = 10000)]
    pub batch_size: usize,

    /// Use freq to fill missing or not
    #[arg(short = 'M', long, default_value = "Impute")]
    pub missing_strategy: String,

    /// if match by id
    #[arg(long, default_value_t = false)]
    pub match_id_flag: bool,

    /// chromosome column for weight file
    #[arg(long, default_value = "CHR")]
    pub chrom: String,

    /// position column for weight file
    #[arg(long, default_value = "POS")]
    pub pos: String,

    /// id column for weight file
    #[arg(long, default_value = "ID")]
    pub snp_id: String,

    /// a1 column for weight file
    #[arg(long, default_value = "A1")]
    pub a1: String,

    /// freq column for weight file
    #[arg(long, default_value = "FREQ")]
    pub freq: String,
}

#[derive(Clone, Debug)]
pub enum MissingStrategy {
    Impute,
    Zero,
    Freq,
}

impl MissingStrategy {
    pub fn new(strategy: &str) -> Result<MissingStrategy> {
        let my_strategy = match strategy {
            "Impute" => MissingStrategy::Impute,
            "Zero" => MissingStrategy::Zero,
            "Freq" => MissingStrategy::Freq,
            _ => {
                return Err(anyhow!(
                    "Argument missing_strategy should be one of the following: [ Impute, Zero, \
                     Freq ], got {}",
                    strategy
                ))
            }
        };
        Ok(my_strategy)
    }
}
