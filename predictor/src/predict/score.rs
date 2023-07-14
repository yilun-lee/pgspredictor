use std::cmp;

use anyhow::Result;
use genoreader::{BedReaderNoLib, ReadGenotype, FreqBedReader};
use ndarray::Array2;
use polars::prelude::DataFrame;

use super::utils::{process_gt, score_to_frame};
use crate::join::weight::Weights;
use crate::join::SWAP;
use crate::meta::MissingStrategy;

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
    let score_frame: DataFrame = score_to_frame(&batch_fam, score, score_names)?;
    Ok(score_frame)
}



pub fn cal_score_array(
    bed: &BedReaderNoLib,
    weights: &Weights,
    iid_idx: &Option<Vec<isize>>,
) -> Result<Array2<f32>> {
    let mut gt= bed.get_geno(&Some(weights.sid_idx.clone()), iid_idx)?;

    // process gt
    process_gt(weights, &mut gt)?;

    // get beta and cal score
    let score = gt.dot(&weights.beta_values);
    Ok(score)
}

pub fn cal_score_array_freq_reader(
    reader: &mut FreqBedReader,
    weights: &Weights,
) -> Result<Array2<f32>> {

    let (stat_vec, freq_vec): (Vec<_>, Vec<_>) = weights.status_freq_vec.iter().cloned().unzip();
    let stat_vec: Vec<bool> = stat_vec.into_iter().map(|x| match x {
        None => false,
        Some(v) => v == SWAP,
    }).collect();

    let gt = match weights.missing_strategy {
        MissingStrategy::Impute => {
            reader.read_snp(&weights.sid_idx, Some(&stat_vec), None)?
        },
        MissingStrategy::Zero => {
            let freq_vec: Vec<f32> = vec![0.;weights.sid_idx.len()];
            reader.read_snp(&weights.sid_idx, Some(&stat_vec), Some(&freq_vec))?
        },
        MissingStrategy::Freq => {
            let freq_vec: Vec<f32> = freq_vec.into_iter().map(|x| x.unwrap_or(0.)).collect();
            reader.read_snp(&weights.sid_idx, Some(&stat_vec), Some(&freq_vec))?
        }
    };

    // get beta and cal score
    let score = gt.dot(&weights.beta_values);
    Ok(score)
}


