use std::{sync::Arc, thread};

use anyhow::Result;
use crossbeam::channel::{unbounded, Receiver, Sender};
use genoreader::BedReaderNoLib;
use log::debug;
use polars::prelude::DataFrame;
use predictor::{
    join::weight::Weights,
    meta::MetaArg,
    predict::{cal_scores, get_empty_score},
};

pub fn cal_score_batch_ind_single(
    meta_arg: &MetaArg,
    weights: Weights,
    bed: BedReaderNoLib,
) -> Result<DataFrame> {
    let mut num_batches = bed.iid_count / meta_arg.batch_size;
    if bed.iid_count % meta_arg.batch_size > 0 {
        num_batches += 1
    }

    let mut result = get_empty_score(meta_arg.get_score_names(false))?;
    for i in 0..num_batches {
        let score = cal_scores(
            &weights,
            i,
            meta_arg.batch_size,
            &bed,
            meta_arg.get_score_names(false),
        )?;
        result = result.vstack(&score)?;
        debug!("Complete {}/{} batch", i + 1, num_batches);
    }
    Ok(result)
}

struct ThreadWorkerBatchInd {
    // batch size
    pub batch_size: usize,
    // re group
    pub bed: Arc<BedReaderNoLib>,
    // some config
    pub weights: Arc<Weights>,
    // recieve from main string, file path
    pub score_names: Arc<Vec<String>>,
    // send from main
    pub receiver: Receiver<Option<usize>>,
    // send to main
    pub sender: Sender<DataFrame>,
}

impl ThreadWorkerBatchInd {
    fn run(&mut self) -> Result<()> {
        while let Some(idx) = self.receiver.recv()? {
            let score: DataFrame = cal_scores(
                &self.weights,
                idx,
                self.batch_size,
                &self.bed,
                &self.score_names,
            )?;
            self.sender.send(score).unwrap();
        }
        Ok(())
    }
}

pub fn cal_score_batch_ind_par(
    meta_arg: &MetaArg,
    weights: Weights,
    bed: BedReaderNoLib,
) -> Result<DataFrame> {
    let (input_sender, input_receiver) = unbounded();
    let (output_sender, output_receiver) = unbounded();

    // init worker
    let weights = Arc::new(weights);
    let bed = Arc::new(bed);
    let score_names = Arc::new(meta_arg.get_score_names(false).clone());

    for _ in 0..meta_arg.thread_num {
        let mut my_worker = ThreadWorkerBatchInd {
            batch_size: meta_arg.batch_size,
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
    let mut num_batches = bed.iid_count / meta_arg.batch_size;
    if bed.iid_count % meta_arg.batch_size > 0 {
        num_batches += 1
    }
    for i in 0..num_batches {
        input_sender.send(Some(i)).unwrap();
    }
    // turn off worker
    for _ in 0..meta_arg.thread_num {
        input_sender.send(None).unwrap();
    }

    // collect result
    let mut init_flag = true;
    let mut result = get_empty_score(&score_names)?;
    for (i, score) in output_receiver.into_iter().enumerate() {
        if init_flag {
            result = score;
            init_flag = false;
        } else {
            result = result.vstack(&score)?;
        }
        debug!("Complete {}/{} batch", i + 1, num_batches);
    }

    Ok(result)
}
