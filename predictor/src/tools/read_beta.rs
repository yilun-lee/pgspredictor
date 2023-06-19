use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{anyhow, Result};
use polars::prelude::{DataType, Field, Schema};

use super::arg::{Args, MissingStrategy};

fn get_schema_table(cli: &Args) -> Result<HashMap<&String, (&str, DataType)>> {
    let mut schema_table = HashMap::new();
    schema_table.insert(&cli.chrom, ("CHR", DataType::Utf8));
    schema_table.insert(&cli.pos, ("POS", DataType::Int32));
    schema_table.insert(&cli.a1, ("A1", DataType::Utf8));

    for i in &cli.score_names {
        schema_table.insert(i, (i, DataType::Float32));
    }

    if let MissingStrategy::Freq = MissingStrategy::new(&cli.missing_strategy)? {
        schema_table.insert(&cli.freq, ("FREQ", DataType::Float32));
    }

    if cli.match_id_flag {
        schema_table.insert(&cli.snp_id, ("ID", DataType::Utf8));
    }

    Ok(schema_table)
}

pub fn get_beta_schema(cli: &Args) -> Result<(Schema, Vec<String>)> {
    // read first line and remove new line
    let file = File::open(&cli.weight_path)?;
    let mut first_line = "".to_string();
    BufReader::new(file).read_line(&mut first_line)?;
    first_line = first_line.replace('\n', "").replace('\r', "");

    // get required col
    let mut schema_table = get_schema_table(cli)?;
    let cols: Vec<String> = schema_table
        .keys()
        .map(|v| v.to_owned().to_owned())
        .collect();

    // generate schema
    let mut field_vec = vec![];
    for i in first_line.split('\t').into_iter() {
        let k = &i.to_owned();
        let (colname, my_datatype) = match schema_table.remove(k) {
            Some(v) => v.clone(),
            None => (i, DataType::Utf8),
        };

        field_vec.push(Field::new(colname, my_datatype))
    }

    // check if there is some column not found
    if schema_table.len() > 0 {
        return Err(anyhow!(
            "Required column not found in beta file {:?}",
            schema_table.keys()
        ));
    }

    Ok((Schema::from_iter(field_vec), cols))
}
