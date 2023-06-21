mod ind_batch;
mod snp_batch;

use std::fs::File;

use anyhow::Result;
use betareader::BetaArg;
use genoreader::BedReaderNoLib;
use ind_batch::{cal_score_batch_ind_par, cal_score_batch_ind_single};
use polars::prelude::{CsvWriter, DataFrame, SerWriter};
use snp_batch::{cal_score_batch_snp_par, cal_score_batch_snp_single};

use crate::{
    args::{Args, MetaArg},
    join::{match_snp, MatchStatus},
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

    pub fn run_batch_ind(&self, bed: BedReaderNoLib) -> Result<(DataFrame, MatchStatus)> {
        let (beta, cols) = self.beta_arg.read()?;
        let (weights, match_status) = match_snp(&self.meta_arg, &cols, &bed.bim, beta)?;

        // run
        let score_frame: DataFrame;
        if self.meta_arg.thread_num == 1 {
            score_frame = cal_score_batch_ind_single(&self.meta_arg, weights, bed)?;
        } else {
            score_frame = cal_score_batch_ind_par(&self.meta_arg, weights, bed)?;
        }
        Ok((score_frame, match_status))
    }

    pub fn run_batch_snp(&self, bed: BedReaderNoLib) -> Result<(DataFrame, MatchStatus)> {
        let (beta_batch_reader, cols) = self.beta_arg.batch_read(self.meta_arg.batch_size)?;

        let score_frame: DataFrame;
        let match_status: MatchStatus;
        if self.meta_arg.thread_num == 1 {
            (score_frame, match_status) =
                cal_score_batch_snp_single(&self.meta_arg, cols, beta_batch_reader, bed)?;
        } else {
            (score_frame, match_status) =
                cal_score_batch_snp_par(&self.meta_arg, cols, beta_batch_reader, bed)?;
        }
        Ok((score_frame, match_status))
    }
}

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
