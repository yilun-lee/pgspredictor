use std::{fs::File, io::{BufReader, BufRead}, sync::Arc};

use polars::{prelude::{DataFrame, Schema, DataType, Field, CsvReader, CsvEncoding, SerReader}, series::Series};
use anyhow::Result;
use std::str;


pub const META_COLS: [&str;2] = ["FID", "IID"];


pub struct CovFrame {
    cov_frame: DataFrame,
    cov_types: Vec<CovType>,
    cov_names: Vec<String>,
}

impl CovFrame {
    pub fn read_cov(cov_path: &str) -> Result<CovFrame>{
        let (my_schmema, cov_names, separater) = get_shema(cov_path)?;
        let cov_frame: DataFrame = CsvReader::from_path(cov_path)?
            .with_delimiter(separater)
            .with_encoding(CsvEncoding::LossyUtf8)
            .with_schema(Arc::new(my_schmema))
            .has_header(true)
            .finish()?;
        let covtypes = preprocess_cov(&cov_frame, &cov_names)?;
        Ok(CovFrame{
            cov_frame, covtypes, cov_names
        })
    }
}


fn get_separater(cov_path: &str) -> u8 {
    if cov_path.contains(".csv"){
        b','
    } else if cov_path.contains(".tsv") {
        b'\t'
    } else {
        b'\t'
    }
}


fn get_shema(cov_path: &str) -> Result<(Schema, Vec<String>, u8)>{
    let sep = get_separater(cov_path);
    let file = File::open(cov_path)?;
    let mut first_line = "".to_string();
    BufReader::new(file).read_line(&mut first_line)?;
    first_line = first_line.replace(['\n', '\r'], "");

    let mut fid_iid_count =0; 
    let mut field_vec = vec![];
    let mut cov_col: Vec<String> = vec![];
    for i in first_line.split(str::from_utf8(&[sep])?){
        if i == META_COLS[0] {
            field_vec.push(Field::new(META_COLS[0], DataType::Utf8));
            fid_iid_count += 1;
        } else if i == META_COLS[1] {
            field_vec.push(Field::new(META_COLS[1], DataType::Utf8));
            fid_iid_count += 1;
        } else {
            field_vec.push(Field::new(i, DataType::Float32));
            cov_col.push(i.to_owned())
        }
    }
    assert!(fid_iid_count==2, "FID or IID not found");

    Ok((Schema::from_iter(field_vec), cov_col, sep))
}


/// drop na, get [CovType]
fn preprocess_cov(cov_frame: &DataFrame, cov_cols: &[String]) -> Result<Vec<CovType>>{
    let mut covtype_vec = vec![];
    for i in cov_cols {
        let cov = cov_frame.column(i)?;
        let cov_type = CovType::from_series(cov)?;
        covtype_vec.push(cov_type);
    }
    Ok(covtype_vec)
}

pub enum CovType {
    Binary,
    Quant
}

impl CovType {
    fn from_series(cov: &Series) -> Result<CovType>{
        let uniq_cov: Vec<f32> = cov.unique()?.f32()?.into_no_null_iter().collect();
        if uniq_cov.contains(&0.) & uniq_cov.contains(&1.) & ( uniq_cov.len() == 2 ){
            Ok(CovType::Binary)
        } else{
            Ok(CovType::Quant)
        }
    }
}
