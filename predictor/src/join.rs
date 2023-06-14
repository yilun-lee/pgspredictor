use core::fmt;

use anyhow::Result;
use ndarray::{s, Array2};
use polars::{
    lazy::dsl::{col, lit, when},
    prelude::{DataFrame, DataFrameJoinOps, IntoLazy, UniqueKeepStrategy},
};
use reader::Beta;

/// this is for display available status
/// https://stackoverflow.com/questions/32710187/how-do-i-get-an-enum-as-a-string
#[derive(Debug)]
pub(crate) enum SnpStatus {
    Good,
    Swap,
    NoMatch,
}
impl fmt::Display for SnpStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// TODO -> deal with duplicated
pub fn match_snp<'a>(
    bim: &DataFrame,
    beta: &Beta,
    score_names: &mut Vec<String>,
) -> Result<DataFrame> {
    let mut cand_col = vec!["CHR".to_string(), "POS".to_string(), "A1".to_string()];
    cand_col.append(score_names);
    let my_beta = &beta.beta.select(cand_col)?;
    let weights = bim
        .select(["IDX", "CHR", "POS", "ALT", "REF"])?
        .inner_join(my_beta, ["CHR", "POS"], ["CHR", "POS"])?;

    let weights = weights
        .lazy()
        .with_column(
            when(col("A1").eq(col("ALT")))
                .then(lit(SnpStatus::Good.to_string()))
                .when(col("A1").eq(col("REF")))
                .then(lit(SnpStatus::Swap.to_string()))
                .otherwise(lit(SnpStatus::NoMatch.to_string()))
                .alias("STATUS"),
        )
        .filter(
            col("STATUS")
                .eq(lit(SnpStatus::NoMatch.to_string().as_str()))
                .not(),
        )
        .unique(
            Some(vec!["CHR".to_string(), "POS".to_string(), "A1".to_string()]),
            UniqueKeepStrategy::First,
        )
        .collect()?;

    Ok(weights)
}
