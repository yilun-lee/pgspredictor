//! This script join Beta and bim by snp to facilitate prediction.
//! Snp will be matched by POS and CHR and make sure A1 from beta is present in
//! REF and ALT of bim file. There may be several snp in beta or bim that shared
//! same position. The first condition is where multiple
//! model snp corresponding to one bfile snp (that is, there are multiple
//! alleles, larger than 2, in the snp). Thanks for the marvelous implementation
//! of [bed-reader](https://github.com/fastlmm/bed-reader), we can just get the specific bfile snp twice, each
//! for one model snp. For other contition, where multiple modle snp
//! corresponding to 1 bfile snp, we just get unique combination of POS, CHR and
//! A1. This is fine since two bfile snp both got the A1 allele and they should
//! be identical in the distribution of A1 allele.
use std::ops::Add;

use anyhow::{anyhow, Result};
use ndarray::Array2;
use polars::{
    lazy::dsl::{all_exprs, col, lit, when},
    prelude::{DataFrame, DataFrameJoinOps, Float32Type, IntoLazy, UniqueKeepStrategy},
};
use serde::Serialize;

use crate::{args::MetaArg, MissingStrategy};

/// constant for SNP match status.
/// [GOOD] indicate that `A1 == ALT`
/// [SWAP] indicate that `A1 == REF`, and genotype need to be swap
/// [NO_MATCH] indicate that `A1 != ALT` && `A1 != REF`, and the snp should be
/// filtered out
pub const GOOD: &str = "Good";
pub const SWAP: &str = "Swap";
pub const NO_MATCH: &str = "NoMatch";

/// Match status, result of the join between bfile and beta.
/// ```rust
/// use crate::join::MatchStatus;
/// let aa = MatchStatus::new_empty();
/// let bb = MatchStatus::new(1234, 234, 198);
/// aa = aa + bb;
/// println("{}", aa);
/// ```
#[derive(Debug, Serialize, Clone)]
pub struct MatchStatus {
    pub bfile_snp: usize,
    pub model_snp: usize,
    pub match_snp: usize,
}

/// init an empty one
#[allow(dead_code)]
impl MatchStatus {
    pub fn new(bfile_snp: usize, model_snp: usize, match_snp: usize) -> MatchStatus {
        MatchStatus {
            bfile_snp: bfile_snp,
            model_snp: model_snp,
            match_snp: match_snp,
        }
    }

    pub fn new_empty() -> MatchStatus {
        MatchStatus {
            bfile_snp: 0,
            model_snp: 0,
            match_snp: 0,
        }
    }
}

/// provide add function
impl Add for MatchStatus {
    type Output = MatchStatus;
    fn add(self, another: MatchStatus) -> MatchStatus {
        MatchStatus {
            bfile_snp: another.bfile_snp,
            model_snp: self.model_snp + another.model_snp,
            match_snp: self.match_snp + another.match_snp,
        }
    }
}

/// match snp function. It do the following
/// 1. Filter Beta by column needed and not null
/// 2. Join Beta and Bim
/// 3. Check swap and keep uniq CHR POS A1 paired
/// 4. Get match status
/// 5. Convert to Weight object for prediction
pub fn match_snp(
    meta_arg: &MetaArg,
    my_cols: &Vec<String>,
    bim: &DataFrame,
    mut beta: DataFrame,
) -> Result<(Weights, MatchStatus, DataFrame)> {
    // filter beta
    // https://stackoverflow.com/questions/76437931/rust-polars-selecting-columns-after-applying-filter-on-rows-of-a-dataframe
    beta = beta
        .select(my_cols)?
        .lazy()
        .filter(all_exprs([col("*").is_not_null()]))
        .collect()?;
    // match by id or chr pos
    let mut matched_beta: DataFrame;
    let identifier_cols: Vec<String>;
    if !meta_arg.match_id_flag {
        matched_beta = bim
            .select(["IDX", "CHR", "POS", "ALT", "REF"])?
            .inner_join(&beta, ["CHR", "POS"], ["CHR", "POS"])?;
        identifier_cols = vec!["CHR".to_string(), "POS".to_string(), "A1".to_string()];
    } else {
        matched_beta =
            bim.select(["IDX", "ID", "ALT", "REF"])?
                .inner_join(&beta, ["ID"], ["ID"])?;
        identifier_cols = vec!["ID".to_string(), "A1".to_string()];
    }

    // filter weights
    matched_beta = matched_beta
        .lazy()
        .with_column(
            when(col("A1").eq(col("ALT")))
                .then(lit(GOOD))
                .when(col("A1").eq(col("REF")))
                .then(lit(SWAP))
                .otherwise(lit(NO_MATCH))
                .alias("STATUS"),
        )
        .filter(col("STATUS").eq(lit(NO_MATCH)).not())
        .unique(Some(identifier_cols), UniqueKeepStrategy::First)
        .collect()?;

    // record match status
    if matched_beta.shape().0 == 0 {
        return Err(anyhow!("No snp matched between models and bfile!"));
    }
    let match_status = MatchStatus {
        bfile_snp: bim.shape().0,
        model_snp: beta.shape().0,
        match_snp: matched_beta.shape().0,
    };
    // create weight object
    let weights_obj = Weights::new(
        matched_beta.clone(),
        meta_arg.score_names,
        meta_arg.missing_strategy,
    )?;
    Ok((weights_obj, match_status, matched_beta))
}

/// Store the matched snp and weight into a Weight obj, which contain and
/// preprocessanything needed for prediction.
#[derive(Clone, Debug)]
pub struct Weights {
    /// The 2d weight matrix
    pub beta_values: Array2<f32>,
    /// snp idx for bed that is matched with beta_values
    pub sid_idx: Vec<isize>,
    /// StATUS and FREQ vec, FREQ may be empty
    pub status_freq_vec: Vec<(Option<String>, Option<f32>)>,
    /// missing strategy for fill missing value
    pub missing_strategy: MissingStrategy,
}

impl Weights {
    fn new(
        mut matched_beta: DataFrame,
        score_names: &Vec<String>,
        missing_strategy: MissingStrategy,
    ) -> Result<Weights> {
        // weights
        let beta_values = matched_beta
            .select(score_names)?
            .to_ndarray::<Float32Type>()?;
        // get sid index in bfile
        let sid_idx: Vec<isize> = matched_beta
            .column("IDX")?
            .u32()?
            .into_no_null_iter()
            .map(|v| v as isize)
            .collect();
        // if no freq, add freq
        if let Err(_) = matched_beta.column("FREQ") {
            matched_beta = matched_beta
                .lazy()
                .with_column(lit(0. as f32).alias("FREQ"))
                .collect()?;
        }
        let freq_iter = matched_beta.column("FREQ")?.f32()?.into_iter();
        let status_freq_vec: Vec<(Option<String>, Option<f32>)> = matched_beta
            .column("STATUS")?
            .utf8()?
            .into_iter()
            .map(|s| {
                match s {
                    Some(s) => Some(s.to_owned()),
                    None => None,
                }
                .to_owned()
            })
            .zip(freq_iter)
            .collect();

        Ok(Weights {
            beta_values,
            sid_idx,
            status_freq_vec,
            missing_strategy,
        })
    }
}
