//! This script contain runner that run prediction for bed files with model
//! It depends [crate::join] and [crate::predict] and run prediction in
//! parallel/single thread wtih batch on snp / ind.

mod ind_batch;
pub mod post;
mod snp_batch;

use anyhow::Result;
use betareader::BetaArg;
use genoreader::BedReaderNoLib;
use ind_batch::{cal_score_batch_ind_par, cal_score_batch_ind_single};
use polars::prelude::DataFrame;
use snp_batch::{cal_score_batch_snp_par, cal_score_batch_snp_single};

use crate::{
    args::{Args, MetaArg},
    join::{match_snp, MatchStatus},
    runner::post::write_beta,
};

/// The [Runner] struct. Basically from [Args]. [BetaArg] is for argument to
/// load weights. [MetaArg] is runner argument such as batch_size.
pub struct Runner<'a> {
    beta_arg: BetaArg<'a>,
    meta_arg: MetaArg<'a>,
}

impl Runner<'_> {
    /// Init from [Args]
    pub fn from_args(cli: &Args) -> Result<Runner> {
        let (beta_arg, meta_arg) = cli.into_struct()?;

        Ok(Runner { beta_arg, meta_arg })
    }

    /// Run batch on sample axis. For single thread ->
    /// [cal_score_batch_ind_single]. For multithread ->
    /// [cal_score_batch_ind_par]
    pub fn run_batch_ind(&self, bed: BedReaderNoLib) -> Result<(DataFrame, MatchStatus)> {
        let (beta, cols) = self.beta_arg.read()?;
        let (weights, match_status, mut match_beta) =
            match_snp(&self.meta_arg, &cols, &bed.bim, beta)?;
        info!(
            "Successful load model. Match {}/{} of snp",
            match_status.match_snp, match_status.model_snp,
        );

        // run
        let score_frame: DataFrame;
        if self.meta_arg.thread_num == 1 {
            score_frame = cal_score_batch_ind_single(&self.meta_arg, weights, bed)?;
        } else {
            score_frame = cal_score_batch_ind_par(&self.meta_arg, weights, bed)?;
        }
        // save beta
        write_beta(&mut match_beta, self.meta_arg.out_prefix, false)?;
        Ok((score_frame, match_status))
    }

    /// Run batch on snp axis. For single thread ->
    /// [cal_score_batch_snp_single]. For multithread ->
    /// [cal_score_batch_snp_par]
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
