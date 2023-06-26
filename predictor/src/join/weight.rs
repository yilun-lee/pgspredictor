use anyhow::Result;
use ndarray::Array2;
use polars::{
    lazy::dsl::lit,
    prelude::{DataFrame, Float32Type, IntoLazy},
};

use crate::meta::MissingStrategy;

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
    /// score names
    pub score_names: Vec<String>,
}

impl Weights {
    pub fn new(
        mut matched_beta: DataFrame,
        score_names: Vec<String>,
        missing_strategy: MissingStrategy,
    ) -> Result<Weights> {
        // weights
        let beta_values = matched_beta
            .select(&score_names)?
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
            score_names,
        })
    }
}
