use std::cmp;

use anyhow::Result;
use polars::prelude::DataFrame;
use reader::{BedReaderNoLib, ReadGenotype};

use super::utils::{process_gt, score_to_frame};
use crate::join::Weights;

pub fn cal_scores(
    weights: &Weights,
    i: usize,
    batch_size: usize,
    bed: &BedReaderNoLib,
    score_names: &Vec<String>,
) -> Result<DataFrame> {
    // cal batch
    let _start = i * batch_size;
    let _end = cmp::min((i + 1) * batch_size, bed.iid_count);
    let iid = Some(bed.iid_idx[_start.._end].to_vec());
    // get gt
    let mut gt = bed.get_geno(&Some(weights.sid_idx.clone()), &iid)?;

    // process gt
    process_gt(&weights, &mut gt)?;

    // get beta and cal score
    let score = gt.dot(&weights.beta_values);

    // get beta and cal score
    let batch_fam = bed.get_ind(&iid, false)?;
    let score_frame = score_to_frame(&batch_fam, score, score_names)?;
    Ok(score_frame)
}
