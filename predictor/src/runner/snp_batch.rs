use std::{sync::Arc, thread};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Sender};
use genoreader::BedReaderNoLib;
//use ndarray::prelude::*;
use polars::prelude::DataFrame;

use crate::{
    join::Weights,
    predict::{cal_scores, get_empty_score},
};

pub fn cal_score_batch_snp_single(
    thread_num: usize,
    batch_size: usize,
    weights: Weights,
    bed: BedReaderNoLib,
    score_names: Vec<String>,
) -> Result<DataFrame> {
    let mut num_batches = bed.iid_count / batch_size;
    if bed.iid_count % batch_size > 0 {
        num_batches += 1
    }

    let mut result = get_empty_score(&score_names)?;
    for i in 0..num_batches {
        let score = cal_scores(&weights, i, batch_size, &bed, &score_names)?;
        result = result.vstack(&score)?;
    }
    Ok(result)
}
