use std::rc::Rc;

use anyhow::Result;
use ndarray::prelude::*;
use polars::{
    lazy::dsl::{col, lit},
    prelude::{DataFrame, Float32Type, IntoLazy},
};
use reader::{BedReader, ReadGenotype};

use super::join::SnpStatus;

fn swap_gt(weights: DataFrame, gt: &mut Array2<f32>) -> Result<()> {
    let swap_idx: Vec<isize> = weights
        .lazy()
        .filter(col("STATUS").eq(lit(SnpStatus::Swap.to_string())))
        .collect()?
        .column("IDX")?
        .u32()?
        .into_no_null_iter()
        .map(|v| v as isize)
        .collect();

    for i in swap_idx {
        gt.slice_mut(s![.., i]).mapv(|v| f32::abs(v - 2.));
    }
    Ok(())
}

fn cal_scores_inner(
    weights: &DataFrame,
    i: usize,
    batch_size: usize,
    bed: &mut BedReader,
    score_names: &Vec<String>,
) -> Result<Array2<f32>> {
    let my_wegihts = weights.slice((i * batch_size) as i64, batch_size);
    let sid: Vec<isize> = my_wegihts
        .column("IDX")?
        .u32()?
        .into_no_null_iter()
        .map(|v| v as isize)
        .collect();

    let mut gt = bed.get_geno(&Some(sid), &None)?;

    let beta_values: ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>> = my_wegihts
        .select(score_names)?
        .to_ndarray::<Float32Type>()?;

    swap_gt(my_wegihts, &mut gt)?;

    let score = gt.dot(&beta_values);
    Ok(score)
}

pub fn cal_scores(
    batch_size: usize,
    weights: &DataFrame,
    bed: &mut BedReader,
    score_names: &Vec<String>,
) -> Result<Array2<f32>> {
    let total_size = weights.shape().0;
    let mut num_batches = total_size / batch_size;
    if total_size % batch_size > 0 {
        num_batches += 1
    }

    let ind_num = bed.fam.shape().0;
    let mut scores = Array::zeros((ind_num, score_names.len()).f());
    for i in 0..num_batches {
        let score = cal_scores_inner(weights, i, batch_size, bed, score_names)?;
        scores += &score;
    }

    Ok(scores)
}
