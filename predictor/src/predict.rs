mod utils;

use std::{sync::Arc, thread};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Sender};
//use ndarray::prelude::*;
use polars::prelude::DataFrame;
use reader::BedReaderNoLib;
use utils::{cal_scores, get_empty_score};

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

    let mut result = get_empty_score(score_names)?;
    if num_batches > 1 {
        for i in 0..num_batches {
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
    // send to main
    pub sender: Sender<DataFrame>,
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
            self.sender.send(score).unwrap();
        }
        Ok(())
    }
}

pub fn cal_scores_par(
    thread_num: usize,
    batch_size: usize,
    weights: DataFrame,
    bed: BedReaderNoLib,
    score_names: Vec<String>,
) -> Result<DataFrame> {
    let (input_sender, input_receiver) = unbounded();
    let (output_sender, output_receiver) = unbounded();

    // init worker
    let weights = Arc::new(weights);
    let bed = Arc::new(bed);
    let score_names = Arc::new(score_names);

    for _ in 0..thread_num {
        let mut my_worker = ThreadWorker {
            batch_size: batch_size,
            bed: bed.clone(),
            weights: weights.clone(),
            score_names: score_names.clone(),
            receiver: input_receiver.clone(),
            sender: output_sender.clone(),
        };
        thread::spawn(move || my_worker.run());
    }

    drop(input_receiver);
    drop(output_sender);

    // send to worker
    let mut num_batches = bed.iid_count / batch_size;
    if bed.iid_count % batch_size > 0 {
        num_batches += 1
    }
    for i in 0..num_batches {
        input_sender.send(Some(i)).unwrap();
    }
    // turn off worker
    for _ in 0..thread_num {
        input_sender.send(None).unwrap();
    }

    // collect result
    let mut init_flag = true;
    let mut result = get_empty_score(&*score_names)?;
    for score in output_receiver {
        if init_flag {
            result = score;
            init_flag = false;
        } else {
            result = result.vstack(&score)?;
        }
    }

    Ok(result)
}
