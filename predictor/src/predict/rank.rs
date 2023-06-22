use anyhow::Result;
use polars::{
    lazy::dsl::{col, lit},
    prelude::{
        DataFrame, DataType, IntoLazy, NamedFromOwned, QuantileInterpolOptions, RankMethod,
        RankOptions,
    },
    series::Series,
};

const RANK_OPT: RankOptions = RankOptions {
    method: RankMethod::Average,
    descending: false,
};

pub fn get_self_percentile(scores: &DataFrame) -> Result<DataFrame> {
    let total_len = scores.shape().0 as i32;
    let percentiles = scores
        .clone()
        .lazy()
        .select([
            col("FID"),
            col("IID"),
            col("*").exclude(["FID", "IID"]).rank(RANK_OPT, None) / lit(total_len),
        ])
        .collect()?;
    Ok(percentiles)
}

pub fn get_pr_table(scores: &DataFrame, score_names: &Vec<String>) -> Result<DataFrame> {
    let my_range: Vec<i32> = (0..101).collect();
    let pr_series = Series::from_vec("PR", my_range);
    let mut my_columns: Vec<Series> = vec![pr_series];

    for score_name in score_names {
        let score = scores.column(score_name)?;
        let mut quantiles = Series::new_empty(score_name, &DataType::Float32);
        for i in 0..101 {
            let i = i as f64 / 100.;
            let quantile = score.quantile_as_series(i, QuantileInterpolOptions::Linear)?;
            quantiles.extend(&quantile)?;
        }
        my_columns.push(quantiles);
    }

    let pr_table = DataFrame::new(my_columns)?;
    Ok(pr_table)
}
