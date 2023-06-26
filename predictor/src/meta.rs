use anyhow::{anyhow, Result};

use crate::join::betahandler::QRange;

/// auto generated column name
pub const STATUS: &str = "STATUS";
pub const RANK: &str = "RANK";

#[derive(Clone, Debug, Copy)]
pub enum MissingStrategy {
    Impute,
    Zero,
    Freq,
}

impl MissingStrategy {
    pub fn new(strategy: &str) -> Result<MissingStrategy> {
        let my_strategy = match strategy {
            "Impute" => MissingStrategy::Impute,
            "Zero" => MissingStrategy::Zero,
            "Freq" => MissingStrategy::Freq,
            _ => {
                return Err(anyhow!(
                    "Argument missing_strategy should be one of the following: [ Impute, Zero, \
                     Freq ], got {}",
                    strategy
                ))
            }
        };
        Ok(my_strategy)
    }
}

#[derive(Clone, Debug)]
pub enum QrangeOrScorenames<'a> {
    QRange(QRange<'a>),
    ScoreNameRaws(&'a Vec<String>),
}

pub struct MetaArg<'a> {
    pub batch_size: usize,
    pub thread_num: usize,
    pub match_id_flag: bool,
    pub missing_strategy: MissingStrategy,
    pub out_prefix: &'a str,
    pub q_range_enum: QrangeOrScorenames<'a>,
}

impl<'a> MetaArg<'a> {
    pub fn get_score_names(&'a self, old_flag: bool) -> &'a Vec<String> {
        match &self.q_range_enum {
            QrangeOrScorenames::QRange(v) => {
                if old_flag {
                    v.score_names_raw
                } else {
                    &v.score_names
                }
            }
            QrangeOrScorenames::ScoreNameRaws(v) => v,
        }
    }
}
