
use anyhow::Result;
use genoreader::meta::PHENO;
use polars::{
    lazy::dsl::{col, lit, pearson_corr, spearman_rank_corr},
    series::Series,
    prelude::{DataFrame, DataType,IntoLazy}
};

/// calculate correlation between score and phenotype
pub fn cal_cor_fn(scores: &DataFrame, score_names: &Vec<&str>) -> Result<DataFrame> {
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

