mod utils;

use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver};
use ndarray::prelude::*;
use polars::prelude::DataFrame;
use reader::BedReader;
use utils::cal_scores;

pub fn cal_scores_onethread(
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
        let score = cal_scores(weights, i, batch_size, bed, score_names)?;
        scores += &score;
    }

    Ok(scores)
}

pub struct ThreadWorker<'a> {
    // batch size
    pub batch_size: usize,
    // re group
    pub bed: Arc<Mutex<&'a mut BedReader>>,
    // some config
    pub weights: Arc<&'a DataFrame>,
    // recieve from main string, file path
    pub score_names: Arc<&'a Vec<String>>,
    // final score
    pub scores: Arc<Mutex<Array2<f32>>>,
    // send from main
    pub receiver: Receiver<Option<usize>>,
}

impl<'a> ThreadWorker<'a> {
    fn run(&mut self) -> Result<()> {
        loop {
            let idx = match self.receiver.recv()? {
                Some(v) => v,
                None => break,
            };

            let mut my_bed = self.bed.lock().unwrap();
            let score = cal_scores(
                *self.weights,
                idx,
                self.batch_size,
                &mut *my_bed,
                *self.score_names,
            )?;

            let mut my_score = self.scores.lock().unwrap();
            *my_score += &score;
        }
        Ok(())
    }
}

pub fn cal_scores_par(
    thread_num: usize,
    batch_size: usize,
    weights: &DataFrame,
    bed: &mut BedReader,
    score_names: &Vec<String>,
) -> Result<Array2<f32>> {
    let (sender, receiver) = unbounded();

    let ind_num = bed.fam.shape().0;

    // init worker
    let weights = Arc::new(weights);
    let bed = Arc::new(Mutex::new(bed));
    let score_names = Arc::new(score_names);
    let scores = Array::zeros((ind_num, score_names.len()).f());
    let scores = Arc::new(Mutex::new(scores));

    for i in 0..thread_num {
        let my_worker = ThreadWorker {
            batch_size: batch_size,
            bed: bed.clone(),
            weights: weights.clone(),
            score_names: score_names.clone(),
            scores: scores.clone(),
            receiver: receiver.clone(),
        };
        thread::spawn(move || my_worker.run());
    }
    let scores = scores;
    let scores = scores.lock().unwrap();
    Ok(*scores)
}
