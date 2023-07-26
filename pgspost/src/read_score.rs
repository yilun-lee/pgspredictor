use std::{fs::File, io::{BufReader, BufRead}, sync::Arc};

use genoreader::meta::PHENO;
use polars::prelude::{DataFrame, Schema, DataType, Field, CsvReader, CsvEncoding, SerReader};
use anyhow::Result;
use std::str;


//use crate::covariate::read_cov::META_COLS;
pub const META_COLS: [&str;2] = ["FID", "IID"];

pub struct PgsScores <'a>{
    pub score: DataFrame,
    pub score_names: Vec<&'a str>,
    pub has_pheno: bool,
}

impl PgsScores <'_>{
    
    pub fn read_score <'a> (score_path: &str, score_names: &'a Vec<String>) -> Result<PgsScores<'a>>{
        let (my_schmema, has_pheno) = PgsScores::get_shema(score_path, score_names)?;
        let score: DataFrame = CsvReader::from_path(score_path)?
            .with_delimiter(b',')
            .with_encoding(CsvEncoding::LossyUtf8)
            .with_schema(Arc::new(my_schmema))
            .has_header(true)
            .finish()?;
        let score_names: Vec<&str> = score_names.iter().map(String::as_str).collect();
        Ok(PgsScores{
            score, score_names, has_pheno
        })
    }

    fn get_shema(score_path: &str, score_names: &[String]) -> Result<(Schema, bool)>{
        let file = File::open(score_path)?;
        let mut first_line = "".to_string();
        BufReader::new(file).read_line(&mut first_line)?;
        first_line = first_line.replace(['\n', '\r'], "");
    
        let mut fid_iid_count =0; 
        let mut field_vec = vec![];
        let mut has_pheno = false;
        for i in first_line.split(","){
            if i == META_COLS[0] {
                field_vec.push(Field::new(META_COLS[0], DataType::Utf8));
                fid_iid_count += 1;
            } else if i == META_COLS[1] {
                field_vec.push(Field::new(META_COLS[1], DataType::Utf8));
                fid_iid_count += 1;
            } else if i == PHENO {
                field_vec.push(Field::new(PHENO, DataType::Float32));
                has_pheno = true;
            } else if score_names.contains(&i.to_owned()) {
                field_vec.push(Field::new(i, DataType::Float32));
            }
        }
        assert!(fid_iid_count==2, "FID or IID not found");
    
        Ok((Schema::from_iter(field_vec), has_pheno))
    }
    
    

}

