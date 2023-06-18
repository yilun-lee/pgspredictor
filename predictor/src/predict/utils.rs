use anyhow::{anyhow, Result};
use ndarray::prelude::*;
use polars::{
    prelude::{DataFrame, DataType, NamedFrom},
    series::Series,
};

use super::super::join::SWAP;
use crate::join::{StatusVec, Weights};

pub fn process_gt(weights: &Weights, gt: &mut Array2<f32>) -> Result<()> {
    match &weights.status_vec {
        StatusVec::SwapIdx(v) => swap_only(v, gt)?,
        StatusVec::StatusFreqVec(v) => swap_and_fillnan(v, gt)?,
    };
    Ok(())
}

// This function swap gt only.
fn swap_only(swap_idx: &Vec<isize>, gt: &mut Array2<f32>) -> Result<()> {
    for i in swap_idx {
        gt.slice_mut(s![.., *i]).mapv_inplace(|v| f32::abs(v - 2.));
    }
    Ok(())
}
// This function swap and fill na in a single walk through of weights
fn swap_and_fillnan(
    status_freq_vec: &Vec<(String, Option<f32>)>,
    gt: &mut Array2<f32>,
) -> Result<()> {
    // https://stackoverflow.com/questions/73318562/how-to-iterate-over-two-different-series-dataframes-and-how-to-access-a-specific

    let mut cc = 0;
    for (status, freq) in status_freq_vec.into_iter() {
        let my_fn: Box<dyn FnMut(f32) -> f32>;
        let freq = match freq {
            Some(v) => *v,
            None => return Err(anyhow!("Got None in series FREQ")),
        };
        // we swap here and fill nan with freq
        if status == SWAP {
            my_fn = Box::new(|x: f32| {
                if x.is_nan() {
                    return freq;
                } else {
                    return f32::abs(x - 2.);
                }
            });
        } else {
            // if no swap -> fill nan with freq only
            my_fn = Box::new(|x: f32| {
                if x.is_nan() {
                    return freq;
                }
                return x;
            });
        }

        gt.slice_mut(s![.., cc]).mapv_inplace(my_fn);
        cc += 1
    }
    Ok(())
}

pub fn score_to_frame(
    fam: &DataFrame,
    score: Array2<f32>,
    score_names: &Vec<String>,
) -> Result<DataFrame> {
    let mut my_columns = vec![fam.column("FID").cloned()?, fam.column("IID").cloned()?];
    for i in 0..score_names.len() {
        let my_score: Vec<f32> = score.slice(s![.., i]).to_vec();
        my_columns.push(Series::new(&score_names[i], my_score))
    }
    let score = DataFrame::new(my_columns)?;
    Ok(score)
}

pub fn get_empty_score(score_names: &Vec<String>) -> Result<DataFrame> {
    let mut my_columns: Vec<Series> = vec![
        Series::new_empty("FID", &DataType::Utf8),
        Series::new_empty("IID", &DataType::Utf8),
    ];
    for i in 0..score_names.len() {
        my_columns.push(Series::new_empty(&score_names[i], &DataType::Float32))
    }
    let score = DataFrame::new(my_columns)?;
    Ok(score)
}
