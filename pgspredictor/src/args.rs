use anyhow::Result;
use betareader::{BetaArg, A1, CHR, FREQ, ID, POS, PVALUE};
use clap::{Args, Parser};
use log::{debug, warn};
use predictor::{
    join::betahandler::QRange,
    meta::{MetaArg, MissingStrategy, QrangeOrScorenames},
};

/// Command argument
#[derive(Parser, Debug)]
#[command(
    name = "pgspredictor.rs", 
    author = "Yilun.lee", 
    version = "0.1.0", 
    about = "A pgs predictor written in rust", long_about = None)]
#[command(next_line_help = true)]
#[command(propagate_version = true)]
pub struct MyArgs {
    /// weight path, should be a tsv file
    pub weight_path: String,

    /// path to plink bed files
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

    /// whether to match by id instead of match by pos and chrom
    #[arg(long, default_value_t = false)]
    pub match_id_flag: bool,

    /// whether to show log, use -v -vv -vvv to present increase log level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// whether to batch by ind, default is batch by snp
    #[arg(long, default_value_t = false)]
    pub batch_ind: bool,

    #[command(flatten)]
    pub beta_col: BetaCol,

    /// Strategy to deal with missing value in genotype. Should be one of the
    /// following: Freq, Impute and Zero
    #[arg(short = 'M', long, default_value = "Impute")]
    pub missing_strategy: String,

    /// whether to write matched snp and related information to *.beta.csv
    #[arg(long, default_value_t = false)]
    pub write_beta: bool,

    /// whether to calculate correlation between PHENO and score
    #[arg(short = 'E', long, default_value_t = false)]
    pub eval_flag: bool,

    /// whether to output percentile and rank
    #[arg(short = 'P', long, default_value_t = false)]
    pub percentile_flag: bool,

    /// path to rank file produce by pgs-predictor. RANK as the first column,
    /// which is 0~100, and the other column are score names. If specified,
    /// percentiles of sample scores are interpolated based on the rank.
    #[arg(short = 'R', long)]
    pub rank_path: Option<String>,

    /// q range file, a headerless tsv file consisted of three columns:
    /// **name**, **from** and **to**, used in filtering p value for
    /// weights.
    #[arg(short = 'Q', long)]
    pub q_ranges: Option<String>,
}

#[derive(Args, Debug)]
pub struct BetaCol {
    /// chromosome column for weight file
    #[arg(long, default_value = CHR)]
    pub chrom: String,

    /// position column for weight file
    #[arg(long, default_value = POS)]
    pub pos: String,

    /// id column for weight file
    #[arg(long, default_value = ID)]
    pub snp_id: String,

    /// a1 column for weight file
    #[arg(long, default_value = A1)]
    pub a1: String,

    /// freq column for weight file
    #[arg(long, default_value = FREQ)]
    pub freq: String,

    /// pvalue column for weight file, required when --q-ranges is specifeid
    #[arg(long, default_value = PVALUE)]
    pub pvalue: String,
}

impl MyArgs {
    /// Convert [MyArgs] into [BetaArg] and [MetaArg]
    /// [BetaArg] is for reading of beta from [betareader]
    pub fn into_struct(&self) -> Result<(BetaArg, MetaArg)> {
        // some check
        let missing_strategy = MissingStrategy::new(&self.missing_strategy)?;
        if matches!(missing_strategy, MissingStrategy::Impute) && self.batch_ind {
            warn!(
                "It is recommended to specify --batch-snp with --missing-strategy \"Impute\". \
                 Since batching on sample cannot calculate complete freq informations. Or you can \
                 use batch larger then sample size."
            )
        }
        debug!("Model: {}", &self.weight_path);
        debug!("Bfile: {}", &self.bed_path);

        let beta_arg = BetaArg {
            // col
            chrom: &self.beta_col.chrom,
            pos: &self.beta_col.pos,
            a1: &self.beta_col.a1,
            freq: &self.beta_col.freq,
            snp_id: &self.beta_col.snp_id,
            pvalue: &self.beta_col.pvalue,
            // misc
            score_names: &self.score_names,
            weight_path: &self.weight_path,
            // flag
            need_freq: matches!(missing_strategy, MissingStrategy::Freq),
            need_id: self.match_id_flag,
            need_pvalue: self.q_ranges.is_some(),
        };
        let qragne_or_score = match &self.q_ranges {
            Some(v) => QrangeOrScorenames::QRange(QRange::new(v, &self.score_names)?),
            None => QrangeOrScorenames::ScoreNameRaws(&self.score_names),
        };
        let meta_arg = MetaArg {
            batch_size: self.batch_size,
            thread_num: self.thread_num,
            match_id_flag: self.match_id_flag,
            missing_strategy,
            out_prefix: &self.out_prefix,
            q_range_enum: qragne_or_score,
        };
        // bed_path and out_path are still only in self, they should not belong to meta
        // and they should only be access in main
        Ok((beta_arg, meta_arg))
    }
}
