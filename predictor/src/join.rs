use core::fmt;

use anyhow::Result;
use polars::{
    lazy::dsl::{col, lit, when},
    prelude::{DataFrame, DataFrameJoinOps, IntoLazy, UniqueKeepStrategy},
};
use reader::Beta;

pub const Good: &str = "Good";
pub const Swap: &str = "Swap";
pub const NoMatch: &str = "NoMatch";

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
                .then(lit(Good))
                .when(col("A1").eq(col("REF")))
                .then(lit(Swap))
                .otherwise(lit(NoMatch))
                .alias("STATUS"),
        )
        .filter(col("STATUS").eq(lit(NoMatch)).not())
        .unique(
            Some(vec!["CHR".to_string(), "POS".to_string(), "A1".to_string()]),
            UniqueKeepStrategy::First,
        )
        .collect()?;

    Ok(weights)
}
