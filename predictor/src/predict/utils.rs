use anyhow::{anyhow, Result};
use genoreader::meta::{FID, IID, PHENO};
use ndarray::prelude::*;
use polars::{
    prelude::{DataFrame, DataType, NamedFrom},
    series::Series,
};

use crate::{
    join::{weight::Weights, SWAP},
    meta::MissingStrategy,
};

fn missing_as_freq(freq: f32, swap_flag: bool) -> Box<dyn FnMut(f32) -> f32> {
    if swap_flag {
        Box::new(move |x: f32| if x.is_nan() { freq } else { 2. - x })
    } else {
        Box::new(move |x: f32| if x.is_nan() { freq } else { x })
    }
}

// This function swap and fill na in a single walk through of weights
pub fn process_gt(weights: &Weights, gt: &mut Array2<f32>) -> Result<()> {
    // https://stackoverflow.com/questions/73318562/how-to-iterate-over-two-different-series-dataframes-and-how-to-access-a-specific

    let mut freq: f32;
    let mut my_fn: Box<dyn FnMut(f32) -> f32>;
    let swap_identifier = &Some(SWAP.to_owned());
    for (cc, (status, default_freq)) in weights.status_freq_vec.iter().enumerate() {
        // use unwrap here since it is unliekly to be None.
        let swap_flag = status == swap_identifier;

        // deal with missing with different strategy
        freq = match weights.missing_strategy {
            MissingStrategy::Zero => 0.,
            MissingStrategy::Freq => match default_freq {
                Some(v) => *v,
                None => return Err(anyhow!("Got None in Series FREQ")),
            },
            MissingStrategy::Impute => {
                // cal non na mean
                let (non_na_count, sum) =
                    gt.slice(s![.., cc])
                        .into_iter()
                        .fold((0_f32, 0_f32), |(mut n, mut s), x| {
                            if !x.is_nan() {
                                n += 1.;
                                s += x;
                            }
                            (n, s)
                        });
                if swap_flag {
                    2. - (sum / non_na_count)
                } else {
                    sum / non_na_count
                }
            }
        };
        // function factory
        my_fn = missing_as_freq(freq, swap_flag);
        // apply on gt
        gt.slice_mut(s![.., cc]).mapv_inplace(my_fn);
    }
    Ok(())
}

pub fn score_to_frame(
    fam: &DataFrame,
    score: Array2<f32>,
    score_names: &[String],
) -> Result<DataFrame> {
    let mut my_columns = vec![
        fam.column(FID).cloned()?,
        fam.column(IID).cloned()?,
        fam.column(PHENO).cloned()?,
    ];
    for (i, score_name) in score_names.iter().enumerate() {
        let my_score: Vec<f32> = score.slice(s![.., i]).to_vec();
        my_columns.push(Series::new(score_name, my_score))
    }
    let score = DataFrame::new(my_columns)?;
    Ok(score)
}

pub fn get_empty_score(score_names: &[String]) -> Result<DataFrame> {
    let mut my_columns: Vec<Series> = vec![
        Series::new_empty(FID, &DataType::Utf8),
        Series::new_empty(IID, &DataType::Utf8),
        Series::new_empty(PHENO, &DataType::Float32),
    ];
    for score_name in score_names {
        my_columns.push(Series::new_empty(score_name, &DataType::Float32))
    }
    let score = DataFrame::new(my_columns)?;
    Ok(score)
}
