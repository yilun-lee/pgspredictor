use anyhow::Result;
use ndarray::Array2;
use polars::{
    lazy::dsl::{col, lit, when},
    prelude::{DataFrame, DataFrameJoinOps, Float32Type, IntoLazy, UniqueKeepStrategy},
};

pub const GOOD: &str = "Good";
pub const SWAP: &str = "Swap";
pub const NO_MATCH: &str = "NoMatch";

/// TODO -> deal with duplicated
pub fn match_snp(
    bim: &DataFrame,
    beta: &DataFrame,
    score_names: &Vec<String>,
    freq_flag: bool,
    cols: &Vec<String>,
) -> Result<Weights> {
    let my_beta = &beta.select(cols)?;
    let weights = bim
        .select(["IDX", "CHR", "POS", "ALT", "REF"])?
        .inner_join(my_beta, ["CHR", "POS"], ["CHR", "POS"])?;

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
        .unique(
            Some(vec!["CHR".to_string(), "POS".to_string(), "A1".to_string()]),
            UniqueKeepStrategy::First,
        )
        .collect()?;

    let weights = Weights::new(weights, score_names, freq_flag)?;
    Ok(weights)
}

#[derive(Clone, Debug)]
pub enum StatusVec {
    SwapIdx(Vec<isize>),
    StatusFreqVec(Vec<(String, Option<f32>)>),
}

#[derive(Clone, Debug)]
pub struct Weights {
    pub beta_values: Array2<f32>,
    pub sid_idx: Vec<isize>,
    pub status_vec: StatusVec,
    pub freq_flag: bool,
}

impl Weights {
    fn new(weights: DataFrame, score_names: &Vec<String>, freq_flag: bool) -> Result<Weights> {
        // weights
        let beta_values = weights.select(score_names)?.to_ndarray::<Float32Type>()?;
        // get sid index in bfile
        let sid_idx: Vec<isize> = weights
            .column("IDX")?
            .u32()?
            .into_no_null_iter()
            .map(|v| v as isize)
            .collect();
        // contain freq or not
        let status_vec = match freq_flag {
            true => {
                let freq_iter = weights.column("FREQ")?.f32()?.into_iter();
                let status_vec: Vec<(String, Option<f32>)> = weights
                    .column("STATUS")?
                    .utf8()?
                    .into_no_null_iter()
                    .map(|s| s.to_owned())
                    .zip(freq_iter)
                    .collect();
                StatusVec::StatusFreqVec(status_vec)
            }
            false => {
                let swap_idx: Vec<isize> = weights
                    .clone()
                    .with_row_count("weight_idx", None)?
                    .lazy()
                    .filter(col("STATUS").eq(lit(SWAP)))
                    .collect()?
                    .column("weight_idx")?
                    .u32()?
                    .into_no_null_iter()
                    .map(|v| v as isize)
                    .collect();
                StatusVec::SwapIdx(swap_idx)
            }
        };
        Ok(Weights {
            beta_values,
            sid_idx,
            status_vec,
            freq_flag,
        })
    }
}
