use anyhow::{anyhow, Result};
use ndarray::Array2;
use polars::{
    lazy::dsl::{col, lit, when},
    prelude::{DataFrame, DataFrameJoinOps, Float32Type, IntoLazy, UniqueKeepStrategy},
};
use serde_json::json;

use crate::MissingStrategy;

pub const GOOD: &str = "Good";
pub const SWAP: &str = "Swap";
pub const NO_MATCH: &str = "NoMatch";

/// TODO -> deal with duplicated
pub fn match_snp(
    bim: &DataFrame,
    beta: &DataFrame,
    score_names: &Vec<String>,
    missing_strategy: MissingStrategy,
    match_snp_flag: bool,
) -> Result<(Weights, serde_json::Value)> {
    // match by id or chr pos
    let weights: DataFrame;
    let identifier_cols: Vec<String>;
    if !match_snp_flag {
        weights = bim
            .select(["IDX", "CHR", "POS", "ALT", "REF"])?
            .inner_join(&beta, ["CHR", "POS"], ["CHR", "POS"])?;
        identifier_cols = vec!["CHR".to_string(), "POS".to_string(), "A1".to_string()];
    } else {
        weights = bim
            .select(["IDX", "ID", "ALT", "REF"])?
            .inner_join(&beta, ["ID"], ["ID"])?;
        identifier_cols = vec!["ID".to_string(), "A1".to_string()];
    }

    // filter weights
    let weights = weights
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
    if weights.shape().0 == 0 {
        return Err(anyhow!("No snp matched between models and bfile!"));
    }
    let match_status = json!({
        "bfile_snp": bim.shape().0,
        "model_snp": beta.shape().0,
        "match_snp": weights.shape().0
    });
    // create weight object
    let weights = Weights::new(weights, score_names, missing_strategy)?;
    Ok((weights, match_status))
}

#[derive(Clone, Debug)]
pub struct Weights {
    pub beta_values: Array2<f32>,
    pub sid_idx: Vec<isize>,
    pub status_freq_vec: Vec<(Option<String>, Option<f32>)>,
    pub missing_strategy: MissingStrategy,
}

impl Weights {
    fn new(
        mut weights: DataFrame,
        score_names: &Vec<String>,
        missing_strategy: MissingStrategy,
    ) -> Result<Weights> {
        // weights
        let beta_values = weights.select(score_names)?.to_ndarray::<Float32Type>()?;
        // get sid index in bfile
        let sid_idx: Vec<isize> = weights
            .column("IDX")?
            .u32()?
            .into_no_null_iter()
            .map(|v| v as isize)
            .collect();
        // if no freq, add freq
        if let Err(_) = weights.column("FREQ") {
            weights = weights
                .lazy()
                .with_column(lit(0. as f32).alias("FREQ"))
                .collect()?;
        }
        let freq_iter = weights.column("FREQ")?.f32()?.into_iter();
        let status_freq_vec: Vec<(Option<String>, Option<f32>)> = weights
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
