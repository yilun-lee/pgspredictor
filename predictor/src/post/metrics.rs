use anyhow::Result;
use genoreader::meta::PHENO;
use ndarray::{Array, Array1};
use polars::{
    lazy::dsl::{col, lit, pearson_corr, spearman_rank_corr},
    prelude::{DataFrame, DataType, IntoLazy},
    series::Series,
};

pub fn pearson_cor_1d(a: &Array1<f32>, b: &Array1<f32>) -> Option<f32> {
    let ll = a.shape()[0];

    let mut a_mean = Array::ones(ll);
    a_mean.fill(a.mean()?);

    let mut b_mean = Array::ones(ll);
    b_mean.fill(b.mean()?);

    let upper_part = (a * b).sum();

    let a2 = &a.mapv(|v| v.powi(2)).sum();
    let b2 = &b.mapv(|v| v.powi(2)).sum();
    let lower_part = a2 * b2;

    let cor = upper_part / lower_part.sqrt();
    Some(cor)
}

pub fn cal_cor(scores: &DataFrame, score_names: &Vec<String>) -> Result<DataFrame> {
    let p_name = "pearson";
    let s_name = "spearman";
    let mut cor_res = DataFrame::new(vec![
        Series::new_empty("Name", &DataType::Utf8),
        Series::new_empty(p_name, &DataType::Float32),
        Series::new_empty(s_name, &DataType::Float32),
    ])?;

    for i in score_names {
        let cor = scores
            .clone()
            .lazy()
            .with_column(lit(i.to_owned()).alias("Name"))
            .groupby(["Name"])
            .agg([
                pearson_corr(col(PHENO), col(i), 0)
                    .cast(DataType::Float32)
                    .alias(p_name),
                spearman_rank_corr(col(PHENO), col(i), 0, true)
                    .cast(DataType::Float32)
                    .alias(s_name),
            ])
            .collect()?
            .select(["Name", p_name, s_name])?;
        cor_res = cor_res.vstack(&cor)?;
    }
    Ok(cor_res)
}
