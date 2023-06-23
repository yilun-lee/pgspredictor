use std::{
    fs::{File, OpenOptions},
    sync::Arc,
};

use anyhow::Result;
use polars::prelude::{
    CsvReader, CsvWriter, DataFrame, DataType, Field, Schema, SerReader, SerWriter,
};

use crate::{
    join::MatchStatus,
    predict::rank::{get_percentile_from_ref, get_pr_table, get_self_percentile, RANK},
};
pub struct PgsResult<'a> {
    scores: &'a mut DataFrame,
    match_status: MatchStatus,
    out_prefix: &'a str,
    score_names: &'a Vec<String>,
    percentile_flag: bool,
    rank_path: &'a Option<String>,
}

impl PgsResult<'_> {
    pub fn new<'a>(
        scores: &'a mut DataFrame,
        match_status: MatchStatus,
        score_names: &'a Vec<String>,
        out_prefix: &'a str,
        percentile_flag: bool,
        rank_path: &'a Option<String>,
    ) -> PgsResult<'a> {
        PgsResult {
            scores,
            match_status,
            out_prefix,
            score_names,
            percentile_flag,
            rank_path,
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
        let out_file: File = File::create(&out_path)?;
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut self.scores)?;
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

    /// output percentile, if rank_path is given, which is in the format
    /// produced in write_rank, used reference rank to interpolate
    /// percentils for the sample scores. Otherwise, calculate the precentiles
    /// according to the current distribution.
    fn write_percentiles(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".percentile.csv";
        let out_file: File = File::create(&out_path)?;
        let mut percentils = match self.rank_path {
            Some(v) => {
                let ref_rank = self.read_rank(v)?;
                info!("Use ref rank to interpolate percentiles");
                get_percentile_from_ref(&self.scores, &ref_rank, self.score_names)?
            }
            None => get_self_percentile(&self.scores)?,
        };
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut percentils)?;
        info!("Output percentiles to {}", &out_path);
        Ok(())
    }

    fn write_rank(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".rank.csv";
        let mut rank = get_pr_table(&self.scores, &self.score_names)?;
        let out_file: File = File::create(&out_path)?;
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut rank)?;
        info!("Output rank to {}", &out_path);
        Ok(())
    }

    fn read_rank(&self, rank_path: &str) -> Result<DataFrame> {
        let mut schema_vec = vec![Field::new(RANK, DataType::Float32)];
        for i in self.score_names {
            schema_vec.push(Field::new(i, DataType::Float32))
        }
        let my_schema = Arc::from(Schema::from_iter(schema_vec));
        let ref_rank: DataFrame = CsvReader::from_path(rank_path)?
            .with_schema(my_schema)
            .has_header(true)
            .finish()?;
        Ok(ref_rank)
    }
}

/// write match beta to file.
/// This function is not in [PgsResult] since ind_batch and snp_batch present
/// different behavior, and therefore this function should be used inside Runner
pub fn write_beta(beta: &mut DataFrame, out_prefix: &str, append_flag: bool) -> Result<()> {
    let out_path = out_prefix.to_owned() + ".beta.tsv";
    let out_file: File;
    if append_flag {
        out_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&out_path)?;
    } else {
        out_file = File::create(&out_path)?;
    }
    CsvWriter::new(out_file)
        .has_header(true)
        .with_delimiter(b'\t')
        .finish(beta)?;
    info!("Output beta to {}", &out_path);
    Ok(())
}
