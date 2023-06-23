use anyhow::{anyhow, Result};

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

pub struct MetaArg<'a> {
    pub score_names: &'a Vec<String>,
    pub batch_size: usize,
    pub thread_num: usize,
    pub match_id_flag: bool,
    pub missing_strategy: MissingStrategy,
    pub out_prefix: &'a str,
}
