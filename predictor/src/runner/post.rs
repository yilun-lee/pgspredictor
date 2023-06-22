use std::fs::File;

use anyhow::Result;
use polars::prelude::{CsvWriter, DataFrame, SerWriter};

use crate::{
    join::MatchStatus,
    predict::rank::{get_pr_table, get_self_percentile},
};
pub struct PgsResult<'a> {
    scores: &'a mut DataFrame,
    match_status: MatchStatus,
    out_prefix: &'a str,
    score_names: &'a Vec<String>,
    percentile_flag: bool,
}

impl PgsResult<'_> {
    pub fn new<'a>(
        scores: &'a mut DataFrame,
        match_status: MatchStatus,
        score_names: &'a Vec<String>,
        out_prefix: &'a str,
        percentile_flag: bool,
    ) -> PgsResult<'a> {
        PgsResult {
            scores,
            match_status,
            out_prefix,
            score_names,
            percentile_flag,
        }
    }

    pub fn write_output(&mut self) -> Result<()> {
        self.write_score()?;
        self.write_status()?;
        if self.percentile_flag {
            self.write_percentiles()?;
            self.write_rank()?;
        }
        Ok(())
    }

    fn write_score(&mut self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".score.csv";
        let out_file: File = File::create(&out_path).unwrap();
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut self.scores)?;
        info!("Output scores to {}", &out_path);
        Ok(())
    }

    fn write_status(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".check.json";
        let json_value = serde_json::to_value(&self.match_status)?;
        let mut file = std::fs::File::create(&out_path)?;
        serde_json::to_writer_pretty(&mut file, &json_value)?;
        info!("Output check status to {}", out_path);
        Ok(())
    }

    fn write_percentiles(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".percentile.csv";
        let mut percentils = get_self_percentile(&self.scores)?;
        let out_file: File = File::create(&out_path).unwrap();
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut percentils)?;
        info!("Output percentiles to {}", &out_path);
        Ok(())
    }

    fn write_rank(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".rank.csv";
        let mut rank = get_pr_table(&self.scores, &self.score_names)?;
        let out_file: File = File::create(&out_path).unwrap();
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut rank)?;
        info!("Output rank to {}", &out_path);
        Ok(())
    }
}
