use std::fs::{File, OpenOptions};
use anyhow::Result;
use log::info;
use polars::prelude::{
    CsvWriter, DataFrame,  SerWriter,
};
use predictor::join::MatchStatus;
pub struct PgsResult<'a> {
    scores: &'a mut DataFrame,
    match_status: MatchStatus,
    out_prefix: &'a str,
}

impl PgsResult<'_> {
    pub fn new<'a>(
        scores: &'a mut DataFrame,
        match_status: MatchStatus,
        out_prefix: &'a str,
    ) -> PgsResult<'a> {
        PgsResult {
            scores,
            match_status,
            out_prefix,
        }
    }

    pub fn write_output(&mut self) -> Result<()> {
        self.write_score()?;
        self.write_status()?;
        Ok(())
    }

    fn write_score(&mut self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".score.csv";
        let out_file: File = File::create(&out_path)?;
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(self.scores)?;
        info!("Output scores to {}", &out_path);
        Ok(())
    }

    fn write_status(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".check.json";
        let json_value = serde_json::to_value(&self.match_status)?;
        let mut file = File::create(&out_path)?;
        serde_json::to_writer_pretty(&mut file, &json_value)?;
        info!("Output check status to {}", out_path);
        Ok(())
    }

}

/// write match beta to file.
/// This function is not in [PgsResult] since ind_batch and snp_batch present
/// different behavior, and therefore this function should be used inside Runner
pub fn write_beta(beta: &mut DataFrame, out_prefix: &str, append_flag: bool) -> Result<()> {
    let out_path = out_prefix.to_owned() + ".beta.tsv";
    let out_file = if append_flag {
        OpenOptions::new()
            .write(true)
            .append(true)
            .open(&out_path)?
    } else {
        File::create(&out_path)?
    };
    CsvWriter::new(out_file)
        .has_header(true)
        .with_delimiter(b'\t')
        .finish(beta)?;
    info!("Output beta to {}", &out_path);
    Ok(())
}
