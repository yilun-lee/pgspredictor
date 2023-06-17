use std::cmp;

use anyhow::Result;
use ndarray::prelude::*;
use polars::{
    lazy::dsl::{col, lit},
    prelude::{DataFrame, Float32Type, IntoLazy, NamedFrom},
    series::Series,
};
use reader::{BedReaderNoLib, ReadGenotype};

use super::super::join::Swap;

fn swap_gt(weights: &DataFrame, gt: &mut Array2<f32>) -> Result<()> {
    let swap_idx: Vec<isize> = weights
        .clone()
        .with_row_count("weight_idx", None)?
        .lazy()
        .filter(col("STATUS").eq(lit(Swap)))
        .collect()?
        .column("weight_idx")?
        .u32()?
        .into_no_null_iter()
        .map(|v| v as isize)
        .collect();

    for i in swap_idx {
        gt.slice_mut(s![.., i]).mapv(|v| f32::abs(v - 2.));
    }
    Ok(())
}

fn score_to_frame(
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

pub fn cal_scores(
    weights: &DataFrame,
    i: usize,
    batch_size: usize,
    bed: &BedReaderNoLib,
    score_names: &Vec<String>,
) -> Result<DataFrame> {
    let sid: Vec<isize> = weights
        .column("IDX")?
        .u32()?
        .into_no_null_iter()
        .map(|v| v as isize)
        .collect();

    let _start = i * batch_size;
    let _end = cmp::min((i + 1) * batch_size, bed.iid_count);
    let iid = Some(bed.iid_idx[_start.._end].to_vec());

    let mut gt = bed.get_geno(&Some(sid), &iid)?;
    let batch_fam = bed.get_ind(&iid, false)?;

    let beta_values = weights.select(score_names)?.to_ndarray::<Float32Type>()?;

    swap_gt(weights, &mut gt)?;

    let score = gt.dot(&beta_values);
    let score_frame = score_to_frame(&batch_fam, score, score_names)?;
    Ok(score_frame)
}
