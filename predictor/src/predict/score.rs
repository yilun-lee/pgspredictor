use std::cmp;

use anyhow::Result;
use genoreader::{BedReaderNoLib, ReadGenotype};
use ndarray::Array2;
use polars::prelude::DataFrame;

use super::utils::{process_gt, score_to_frame};
use crate::join::weight::Weights;

pub fn cal_scores(
    weights: &Weights,
    i: usize,
    batch_size: usize,
    bed: &BedReaderNoLib,
    score_names: &[String],
) -> Result<DataFrame> {
    // cal batch
    let _start = i * batch_size;
    let _end = cmp::min((i + 1) * batch_size, bed.iid_count);
    let iid = Some(bed.iid_idx[_start.._end].to_vec());
    // get gt
    let score = cal_score_array(bed, weights, &iid)?;

    // get beta and cal score
    let batch_fam = bed.get_ind(&iid, false)?;
    let score_frame = score_to_frame(&batch_fam, score, score_names)?;
    Ok(score_frame)
}

pub fn cal_score_array(
    bed: &BedReaderNoLib,
    weights: &Weights,
    iid_idx: &Option<Vec<isize>>,
) -> Result<Array2<f32>> {
    let mut gt = bed.get_geno(&Some(weights.sid_idx.clone()), iid_idx)?;

    // process gt
    process_gt(weights, &mut gt)?;

    // get beta and cal score
    let score = gt.dot(&weights.beta_values);
    Ok(score)
}
