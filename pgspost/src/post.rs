mod rank;
mod metrics;

use std::{
    fs::File,
    sync::Arc,
};
use polars::prelude::{CsvReader, CsvWriter, DataFrame, DataType, Field, Schema, SerReader, SerWriter};
use anyhow::Result;
use log::info;

use predictor::meta::RANK;
use crate::read_score::PgsScores;
use rank::{get_percentile_from_ref, get_pr_table, get_self_percentile};

pub struct PgsPost<'a> {
    pgs_scores: &'a PgsScores<'a>,
    out_prefix: &'a str,
    rank_path: &'a Option<String>,
    eval: bool,
}

impl PgsPost<'_> {
    pub fn new<'a>(
        pgs_scores: &'a PgsScores<'a>,
        out_prefix: &'a str,
        rank_path: &'a Option<String>,
        eval: bool,
    ) -> PgsPost<'a> {
        PgsPost {
            pgs_scores,
            out_prefix,
            rank_path,
            eval,
        }
    }

    pub fn write_output(&mut self) -> Result<()> {
        if self.eval {
            self.cal_cor()?;
        }
        self.write_percentiles()?;
        self.write_rank()?;
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
                get_percentile_from_ref(&self.pgs_scores.score, &ref_rank, &self.pgs_scores.score_names)?
            }
            None => get_self_percentile(&self.pgs_scores.score)?,
        };
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut percentils)?;
        info!("Output percentiles to {}", &out_path);
        Ok(())
    }

    fn write_rank(&self) -> Result<()> {
        let out_path = self.out_prefix.to_owned() + ".rank.csv";
        let mut rank = get_pr_table(&self.pgs_scores.score, &self.pgs_scores.score_names)?;
        let out_file: File = File::create(&out_path)?;
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut rank)?;
        info!("Output rank to {}", &out_path);
        Ok(())
    }

    fn read_rank(&self, rank_path: &str) -> Result<DataFrame> {
        let mut schema_vec = vec![Field::new(RANK, DataType::Float32)];
        for i in &self.pgs_scores.score_names {
            schema_vec.push(Field::new(i, DataType::Float32))
        }
        let my_schema = Arc::from(Schema::from_iter(schema_vec));
        let ref_rank: DataFrame = CsvReader::from_path(rank_path)?
            .with_schema(my_schema)
            .has_header(true)
            .finish()?;
        Ok(ref_rank)
    }

    fn cal_cor(&self) -> Result<()> {
        assert!(&self.pgs_scores.has_pheno, "No pheno column found");
        let out_path = self.out_prefix.to_owned() + ".cor.csv";
        let mut cor_res = metrics::cal_cor_fn(&self.pgs_scores.score, &self.pgs_scores.score_names).unwrap();
        let out_file: File = File::create(&out_path)?;
        CsvWriter::new(out_file)
            .has_header(true)
            .finish(&mut cor_res)?;
        info!("Output correlation to {}", &out_path);
        Ok(())
    }
}
