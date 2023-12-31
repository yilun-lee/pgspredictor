use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use polars::{
    io::mmap::MmapBytesReader,
    prelude::{
        read_impl::OwnedBatchedCsvReader, CsvEncoding, CsvReader, DataFrame, DataType, Field,
        Schema, SerReader,
    },
};

/// const for default column name
pub const ID: &str = "ID";
pub const CHR: &str = "CHR";
pub const POS: &str = "POS";
pub const A1: &str = "A1";
/// optional default column name
pub const FREQ: &str = "FREQ";
pub const PVALUE: &str = "P";
pub const RANK: &str = "RANK";

#[derive(Debug)]
pub struct BetaArg<'a> {
    // col
    pub chrom: &'a str,
    pub pos: &'a str,
    pub a1: &'a str,
    pub freq: &'a str,
    pub snp_id: &'a str,
    pub pvalue: &'a str,
    // misc
    pub score_names: &'a Vec<String>,
    pub weight_path: &'a str,
    // flag
    pub need_freq: bool,
    pub need_id: bool,
    pub need_pvalue: bool,
}

impl<'a> BetaArg<'a> {
    fn get_schema_table(&self) -> Result<HashMap<&str, (&str, DataType)>> {
        let mut schema_table = HashMap::new();
        schema_table.insert(self.chrom, (CHR, DataType::Utf8));
        schema_table.insert(self.pos, (POS, DataType::Int32));
        schema_table.insert(self.a1, (A1, DataType::Utf8));

        for i in self.score_names {
            schema_table.insert(i, (i, DataType::Float32));
        }

        if self.need_freq {
            schema_table.insert(self.freq, (FREQ, DataType::Float32));
        }

        if self.need_id {
            schema_table.insert(self.snp_id, (ID, DataType::Utf8));
        }

        if self.need_pvalue {
            schema_table.insert(self.pvalue, (PVALUE, DataType::Float32));
        }

        Ok(schema_table)
    }

    pub fn get_beta_schema(&self) -> Result<(Schema, Vec<String>)> {
        // read first line and remove new line
        let file = File::open(self.weight_path)?;
        let mut first_line = "".to_string();
        BufReader::new(file).read_line(&mut first_line)?;
        first_line = first_line.replace(['\n', '\r'], "");

        // get required col
        let mut schema_table = self.get_schema_table()?;
        let cols: Vec<String> = schema_table
            .keys()
            .map(|v| v.to_owned().to_owned())
            .collect();

        // generate schema
        let mut field_vec = vec![];
        for i in first_line.split('\t') {
            let (colname, my_datatype) = match schema_table.remove(i) {
                Some(v) => v.clone(),
                None => (i, DataType::Utf8),
            };
            field_vec.push(Field::new(colname, my_datatype))
        }

        // check if there is some column not found
        if !schema_table.is_empty() {
            return Err(anyhow!(
                "Required column not found in beta file {:?}",
                schema_table.keys()
            ));
        }

        Ok((Schema::from_iter(field_vec), cols))
    }

    pub fn batch_read(&self, mut batch_size: usize) -> Result<(OwnedBatchedCsvReader, Vec<String>)> {
        let (my_schmema, cols) = self.get_beta_schema()?;
        let my_schmema = Arc::new(my_schmema);
        // https://github.com/pola-rs/polars/blob/main/py-polars/src/batched_csv.rs
        let file = File::open(self.weight_path)?;
        // make sure batch_size > line number 
        let tmp_beta: DataFrame = CsvReader::from_path(self.weight_path)?
            .with_delimiter(b'\t')
            .with_encoding(CsvEncoding::LossyUtf8)
            .with_schema(my_schmema.clone())
            .with_n_rows(Some(batch_size))
            .has_header(true)
            .finish()?;
        if tmp_beta.height() < batch_size{
            batch_size = tmp_beta.height() ;
        }

        let reader = Box::new(file) as Box<dyn MmapBytesReader>;
        let reader: OwnedBatchedCsvReader = CsvReader::new(reader)
            .with_chunk_size(batch_size)
            .with_delimiter(b'\t')
            .with_encoding(CsvEncoding::LossyUtf8)
            .has_header(true)
            .batched_read(Some(my_schmema))?;
        Ok((reader, cols))
    }

    pub fn read(&self) -> Result<(DataFrame, Vec<String>)> {
        let (my_schmema, cols) = self.get_beta_schema()?;
        let beta: DataFrame = CsvReader::from_path(self.weight_path)?
            .with_delimiter(b'\t')
            .with_encoding(CsvEncoding::LossyUtf8)
            .with_schema(Arc::new(my_schmema))
            .has_header(true)
            .finish()?;

        Ok((beta, cols))
    }
}
