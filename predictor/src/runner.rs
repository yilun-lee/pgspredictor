mod ind_batch;
mod snp_batch;

use std::fs::File;

use anyhow::Result;
use betareader::BetaArg;
use genoreader::BedReaderNoLib;
pub use ind_batch::{cal_score_batch_ind_par, cal_score_batch_ind_single};
use polars::prelude::{CsvWriter, DataFrame, SerWriter};
use serde_json::Value;

use crate::{
    args::{Args, MetaArg},
    join::match_snp,
};

pub struct Runner<'a> {
    beta_arg: BetaArg<'a>,
    meta_arg: MetaArg<'a>,
}

impl Runner<'_> {
    pub fn from_args(cli: &Args) -> Result<Runner> {
        let (beta_arg, meta_arg) = cli.into_struct()?;

        Ok(Runner { beta_arg, meta_arg })
    }

    pub fn run_batch_ind(&self, bed: BedReaderNoLib) -> Result<(DataFrame, Value)> {
        let (beta, cols) = self.beta_arg.read()?;
        let (weights, match_status) = match_snp(
            &cols,
            &bed.bim,
            beta,
            &self.meta_arg.score_names,
            self.meta_arg.missing_strategy.clone(),
            self.meta_arg.match_id_flag,
        )?;

        // run
        let scores: DataFrame;
        if self.meta_arg.thread_num == 1 {
            scores = cal_score_batch_ind_single(
                self.meta_arg.thread_num,
                self.meta_arg.batch_size,
                weights,
                bed,
                self.meta_arg.score_names,
            )?;
        } else {
            scores = cal_score_batch_ind_par(
                self.meta_arg.thread_num,
                self.meta_arg.batch_size,
                weights,
                bed,
                self.meta_arg.score_names,
            )?;
        }
        Ok((scores, match_status))
    }
}

fn snp_batch_runner() {}
pub fn write_file(out_path: &str, scores: &mut DataFrame) -> Result<()> {
    let out_path: File = File::create(out_path).unwrap();
    CsvWriter::new(out_path).has_header(true).finish(scores)?;
    Ok(())
}

pub fn save_as_json(my_struct: serde_json::Value, out_path: &str) -> Result<()> {
    let mut file = std::fs::File::create(out_path)?;
    serde_json::to_writer_pretty(&mut file, &my_struct)?;
    Ok(())
}
