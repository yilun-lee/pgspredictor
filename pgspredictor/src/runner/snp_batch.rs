use std::{sync::Arc, thread};

use anyhow::{anyhow, Result};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use genoreader::{BfileSet, FreqBedReader};
use log::debug;
use ndarray::Array2;
//use ndarray::prelude::*;
use polars::prelude::{read_impl::OwnedBatchedCsvReader, DataFrame};
use predictor::{
    join::{match_snp, weight::Weights, MatchStatus},
    meta::MetaArg,
    predict::{cal_score_array_freq_reader, score_to_frame},
};

use crate::runner::post::write_beta;

#[allow(unused_variables)]
pub fn cal_score_batch_snp_single(
    meta_arg: &MetaArg,
    cols: Vec<String>,
    mut beta_batch_reader: OwnedBatchedCsvReader,
    bfileset: BfileSet,
    write_match: bool,
) -> Result<(DataFrame, MatchStatus)> {
    // to avoid of binding
    let mut beta: DataFrame;
    let mut matched_beta: DataFrame;
    let mut new_match_status: MatchStatus;
    let mut weights: Weights;
    let score_names: Vec<String>;
    // init
    let mut match_status = MatchStatus::new_empty();
    let mut score_sum: Option<Array2<f32>> = None;
    let mut i = 0;

    let mut geno_reader = FreqBedReader::new(Arc::new(bfileset))?;
    loop {
        // get beta
        beta = match beta_batch_reader.next_batches(1)? {
            Some(v) => v[0].to_owned(),
            None => break,
        };
        beta = beta.select(&cols)?;
        // match snp
        (weights, new_match_status, matched_beta) = match match_snp(meta_arg, &cols, &geno_reader.bfile_set.bim, beta)
        {
            Ok(v) => v,
            // TODO -> Classify Error
            Err(e) => {
                debug!("{}", e);
                continue;
            }
        };
        // add match_status
        match_status = match_status + new_match_status;
        // cal score
        let score = cal_score_array_freq_reader(&mut geno_reader, &weights)?;
        score_sum = match score_sum {
            Some(v) => Some(v + score),
            None => Some(score),
        };
        // write beta
        if i == 0 {
            write_beta(&mut matched_beta, meta_arg.out_prefix, false)?;
        } else {
            write_beta(&mut matched_beta, meta_arg.out_prefix, true)?;
        }
        debug!("Complete {} batch", i + 1);
        i += 1;
    }
    // unwrap score
    let score_sum = match score_sum {
        Some(v) => v,
        None => return Err(anyhow!("score_sum is not initialized")),
    };
    // score for frame
    let batch_fam = geno_reader.bfile_set.get_ind(None, false)?;
    let score_frame = score_to_frame(&batch_fam, score_sum, meta_arg.get_score_names(false))?;

    Ok((score_frame, match_status))
}

#[allow(clippy::redundant_allocation)]
pub struct ThreadWorkerBatchSnp<'a> {
    // batch size
    pub batch_size: usize,
    // re group
    pub bfileset: Arc<BfileSet>,
    // recieve from main string, file path
    pub cols: Arc<Vec<String>>,
    // recieve from main string, file path
    pub meta_arg: Arc<&'a MetaArg<'a>>,
    // get from main
    pub receiver: Receiver<Option<DataFrame>>,
    // send to main
    pub sender: Sender<(Array2<f32>, MatchStatus, DataFrame)>,
}

impl ThreadWorkerBatchSnp<'_> {
    fn run(&mut self) -> Result<()> {
        let mut beta: DataFrame;
        let mut i = 0;

        let mut geno_reader = FreqBedReader::new(self.bfileset.clone())?;
        loop {
            beta = match self.receiver.recv()? {
                Some(v) => v,
                None => break,
            };
            beta = beta.select(&*self.cols)?;
            // match snp
            let (weights, match_status, matched_beta) =
                match_snp(&self.meta_arg, &self.cols, &geno_reader.bfile_set.bim, beta)?;
            // cal score
            let score = cal_score_array_freq_reader(&mut geno_reader, &weights)?;
            self.sender
                .send((score, match_status, matched_beta))
                .unwrap();
            debug!("Complete {} batch", i + 1);
            i += 1;
        }
        Ok(())
    }
}

type ThreadResVec<'a> = Vec<thread::ScopedJoinHandle<'a, Result<()>>>;
pub fn cal_score_batch_snp_par(
    meta_arg: &MetaArg,
    cols: Vec<String>,
    mut beta_batch_reader: OwnedBatchedCsvReader,
    bfileset: BfileSet,
    write_match: bool,
) -> Result<(DataFrame, MatchStatus)> {
    let (input_sender, input_receiver) = bounded(meta_arg.thread_num * 2);
    let (output_sender, output_receiver) = unbounded();


    // init worker
    let cols: Arc<Vec<String>> = Arc::new(cols);
    let meta_arg = Arc::new(meta_arg.clone());
    let bfileset = Arc::new(bfileset);

    let (score_sum, match_status) = thread::scope(|scope| -> Result<(Array2<f32>, MatchStatus)> {
        let mut thread_vec: ThreadResVec = vec![];
        for _ in 0..meta_arg.thread_num {
            let mut my_worker = ThreadWorkerBatchSnp {
                batch_size: meta_arg.batch_size,
                bfileset: bfileset.clone(),
                cols: cols.clone(),
                meta_arg: meta_arg.clone(),
                receiver: input_receiver.clone(),
                sender: output_sender.clone(),
            };

            thread_vec.push(scope.spawn(move || my_worker.run()));
        }

        drop(input_receiver);
        drop(output_sender);

        // send to worker
        let mut beta: DataFrame;
        loop {
            beta = match beta_batch_reader.next_batches(1)? {
                Some(v) => v[0].to_owned(),
                None => break,
            };
            input_sender.send(Some(beta)).unwrap();
        }
        // end the input_sender, which will terminate the output_sender and threads
        for _ in 0..meta_arg.thread_num {
            input_sender.send(None).unwrap();
        }

        // collect result untils output_sender is terminated
        let (score_sum, match_status) =
            join_threads_collect_result(output_receiver, meta_arg.out_prefix, write_match)?;
        // join
        join_thread_vec(thread_vec)?;

        Ok((score_sum, match_status))
    })?;

    // score to dataframe
    let batch_fam = bfileset.get_ind(None, false)?;
    let score_frame = score_to_frame(&batch_fam, score_sum, meta_arg.get_score_names(false))?;

    Ok((score_frame, match_status))
}

fn join_threads_collect_result(
    output_receiver: Receiver<(Array2<f32>, MatchStatus, DataFrame)>,
    out_prefix: &str,
    write_match: bool,
) -> Result<(Array2<f32>, MatchStatus)> {
    let mut match_status = MatchStatus::new_empty();
    let mut score_sum: Option<Array2<f32>> = None;
    for (cc, (score, new_match_status, mut matched_beta)) in output_receiver.into_iter().enumerate()
    {
        // add match_status
        match_status = match_status + new_match_status;
        // cal score
        score_sum = match score_sum {
            Some(v) => Some(v + score),
            None => Some(score),
        };
        // write beta
        if write_match {
            if cc == 0 {
                write_beta(&mut matched_beta, out_prefix, false)?;
            } else {
                write_beta(&mut matched_beta, out_prefix, true)?;
            }
        }
    }
    // unwrap score
    let score_sum = match score_sum {
        Some(v) => v,
        None => return Err(anyhow!("score_sum is not initialized")),
    };
    Ok((score_sum, match_status))
}

fn join_thread_vec(thread_vec: ThreadResVec) -> Result<()> {
    for i in thread_vec {
        i.join().unwrap()?;
    }
    Ok(())
}
