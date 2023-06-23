use anyhow::Result;
use betareader::BetaArg;
use clap::Parser;
use log::{debug, warn};
use predictor::meta::{MetaArg, MissingStrategy};

/// Command argument
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
    pub out_prefix: String,

    /// score names: scores to be process
    #[arg(short = 'n', long)]
    pub score_names: Vec<String>,

    /// number of thread to run
    #[arg(short = 'T', long, default_value_t = 1)]
    pub thread_num: usize,

    /// batch size for sample / or snp if batch-snp flag is set.
    #[arg(short = 'B', long, default_value_t = 10000)]
    pub batch_size: usize,

    /// Strategy to deal with missing value in genotype. Should be one of the
    /// following: Freq, Impute and Zero
    #[arg(short = 'M', long, default_value = "Impute")]
    pub missing_strategy: String,

    /// whether to match by id instead of match by pos and chrom
    #[arg(long, default_value_t = false)]
    pub match_id_flag: bool,

    /// whether show log
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// whether to batch by snp, default is batch by ind
    #[arg(long, default_value_t = false)]
    pub batch_snp: bool,

    /// whether to output percentile and rank
    #[arg(short = 'P', long, default_value_t = false)]
    pub percentile_flag: bool,

    /// path to rank file produce by pgs-predictor. RANK as the first column,
    /// which is 0~100, and the other column are score names. If specified,
    /// percentiles of sample scores are interpolated based on the rank.
    #[arg(short = 'r', long)]
    pub rank_path: Option<String>,

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

impl Args {
    /// Convert [Args] into [BetaArg] and [MetaArg]
    /// [BetaArg] is for reading of beta from [betareader]
    pub fn into_struct(&self) -> Result<(BetaArg, MetaArg)> {
        // some check
        let missing_strategy = MissingStrategy::new(&self.missing_strategy)?;
        if matches!(missing_strategy, MissingStrategy::Impute) && !self.batch_snp {
            warn!(
                "It is recommended to specify --batch-snp with --missing-strategy \"Impute\". \
                 Since batching on sample cannot calculate complete freq informations. Or you can \
                 use batch larger then sample size."
            )
        }
        debug!("Model: {}", &self.weight_path);
        debug!("Bfile: {}", &self.bed_path);

        let beta_arg = BetaArg {
            chrom: &self.chrom,
            pos: &self.pos,
            a1: &self.a1,
            freq: &self.freq,
            snp_id: &self.snp_id,
            score_names: &self.score_names,
            weight_path: &self.weight_path,
            need_freq: matches!(missing_strategy, MissingStrategy::Freq),
            need_id: self.match_id_flag,
        };
        let meta_arg = MetaArg {
            score_names: &self.score_names,
            batch_size: self.batch_size,
            thread_num: self.thread_num,
            match_id_flag: self.match_id_flag,
            missing_strategy: missing_strategy,
            out_prefix: &self.out_prefix,
        };
        // bed_path and out_path are still only in self, they should not belong to meta
        // and they should only be access in main
        Ok((beta_arg, meta_arg))
    }
}
