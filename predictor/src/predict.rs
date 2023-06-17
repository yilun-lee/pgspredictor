mod utils;

use std::{sync::Arc, thread};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver};
//use ndarray::prelude::*;
use polars::prelude::{concat, DataFrame};
use reader::BedReaderNoLib;
use utils::cal_scores;

pub fn cal_scores_onethread(
    batch_size: usize,
    weights: &DataFrame,
    bed: &BedReaderNoLib,
    score_names: &Vec<String>,
) -> Result<DataFrame> {
    let mut num_batches = bed.iid_count / batch_size;
    if bed.iid_count % batch_size > 0 {
        num_batches += 1
    }

    let mut result = cal_scores(weights, 0, batch_size, bed, score_names)?;
    if num_batches > 1 {
        for i in 1..num_batches {
            let score = cal_scores(weights, i, batch_size, bed, score_names)?;
            result = result.vstack(&score)?;
        }
    }
    Ok(result)
}

pub struct ThreadWorker {
    // batch size
    pub batch_size: usize,
    // re group
    pub bed: Arc<BedReaderNoLib>,
    // some config
    pub weights: Arc<DataFrame>,
    // recieve from main string, file path
    pub score_names: Arc<Vec<String>>,
    // send from main
    pub receiver: Receiver<Option<usize>>,
}

impl<'a> ThreadWorker {
    fn run(&mut self) -> Result<()> {
        loop {
            let idx = match self.receiver.recv()? {
                Some(v) => v,
                None => break,
            };

            let score: DataFrame = cal_scores(
                &*self.weights,
                idx,
                self.batch_size,
                &*self.bed,
                &*self.score_names,
            )?;
        }
        Ok(())
    }
}
/*
pub fn cal_scores_par(
    thread_num: usize,
    batch_size: usize,
    weights: DataFrame,
    bed: BedReaderNoLib,
    score_names: Vec<String>,
) -> Result<Array2<f32>> {
    let (sender, receiver) = unbounded();

    let ind_num = bed.fam.shape().0;

    // init worker
    let weights = Arc::new(weights);
    let bed = Arc::new(bed);
    let score_names = Arc::new(score_names);

    for i in 0..thread_num {
        let mut my_worker = ThreadWorker {
            batch_size: batch_size,
            bed: bed.clone(),
            weights: weights.clone(),
            score_names: score_names.clone(),
            receiver: receiver.clone(),
        };
        thread::spawn(move || my_worker.run());
    }
    Ok(scores)
}
*/
