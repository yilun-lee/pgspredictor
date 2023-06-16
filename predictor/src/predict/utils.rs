use anyhow::Result;
use ndarray::prelude::*;
use polars::{
    lazy::dsl::{col, lit},
    prelude::{DataFrame, Float32Type, IntoLazy},
};
use reader::{BedReader, ReadGenotype};

use super::super::join::SnpStatus;

pub fn swap_gt(weights: DataFrame, gt: &mut Array2<f32>) -> Result<()> {
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

pub fn cal_scores(
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
