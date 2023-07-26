use anyhow::Result;
use betareader::RANK;
use genoreader::meta::{FID, IID};
use interp::interp_slice;
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
            col(FID),
            col(IID),
            col("*").exclude([FID, IID]).rank(RANK_OPT, None) / lit(total_len),
        ])
        .collect()?;
    Ok(percentiles)
}

pub fn get_pr_table(scores: &DataFrame, score_names: &Vec<&str>) -> Result<DataFrame> {
    let my_range: Vec<i32> = (0..101).collect();
    let pr_series = Series::from_vec(RANK, my_range);
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

pub fn get_percentile_from_ref(
    scores: &DataFrame,
    ref_rank: &DataFrame,
    score_names: &Vec<&str>,
) -> Result<DataFrame> {
    let quantiles: Vec<f32> = ref_rank.column(RANK)?.f32()?.into_no_null_iter().collect();
    let mut series_vec = vec![scores.column(FID)?.clone(), scores.column(IID)?.clone()];
    for i in score_names {
        let score: Vec<f32> = scores.column(i)?.f32()?.into_no_null_iter().collect();
        let rank: Vec<f32> = ref_rank.column(i)?.f32()?.into_no_null_iter().collect();
        let percentile = interp_slice(&rank, &quantiles, &score);
        series_vec.push(Series::from_vec(i, percentile));
    }
    let percentiles = DataFrame::new(series_vec)?;
    Ok(percentiles)
}
