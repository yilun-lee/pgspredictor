mod test;

use std::sync::Arc;

use anyhow::Result;
use polars::prelude::{CsvReader, DataFrame, DataType, Field, Schema, SerReader};

pub struct BetaMeta {}

#[allow(dead_code)]
pub struct Beta {
    pub beta_path: String,
    pub beta: DataFrame,
    pub snp_count: usize,
    pub meta: BetaMeta,
}

#[allow(dead_code)]
impl Beta {
    pub fn new(beta_path: &str) -> Result<Beta> {
        let my_schmema = Beta::get_schema("Lassosum");
        let beta = CsvReader::from_path(beta_path)?
            .with_delimiter(b'\t')
            .with_schema(Arc::new(my_schmema))
            .has_header(true)
            .finish()?;
        let cc = beta.shape().0;
        Ok(Beta {
            beta_path: beta_path.to_owned(),
            beta,
            snp_count: cc,
            meta: BetaMeta {},
        })
    }

    fn get_schema(score_name: &str) -> Schema {
        Schema::from_iter(
            vec![
                Field::new("CHR", DataType::Utf8),
                Field::new("POS", DataType::Int32),
                Field::new("ID", DataType::Utf8),
                Field::new("REF", DataType::Utf8),
                Field::new("ALT", DataType::Utf8),
                Field::new("A1", DataType::Utf8),
                Field::new(score_name, DataType::Float32),
            ]
            .into_iter(),
        )
    }
}

fn _get_beta_schema() -> Schema {
    Schema::from_iter(
        vec![
            Field::new("CHR", DataType::Utf8),
            Field::new("POS", DataType::UInt8),
            Field::new("A1", DataType::Utf8),
            Field::new("SCORE", DataType::Float32),
        ]
        .into_iter(),
    )
}
