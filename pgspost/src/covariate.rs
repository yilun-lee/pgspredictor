//! Calculate covriate weights
//! Input: Scores([DataFrame]) and covriate path ([str])
//! Mode: Fit and Predict
//! Fit Step:
//! - Load and check covriate
//! - Standardize
//! - Linear fit
//! - Evaluate
//! - Save weights and output score
//! Predict Step:
//! - Load weights
//! - Linear predict
//! - Output score 

mod fit;
pub mod read_cov;

use ndarray::{Array2,Array1};
use polars::prelude::DataFrame;
use anyhow::Result;

use self::fit::{FeatureSet, WeightSet};

/// This is the main interface for cov pipeline
pub trait RunCov {
    fn run_cov_pipeline(score_frame: &DataFrame, score_names: &[String]) -> Result<DataFrame>;
}

enum CovAnalysisMode {
    Fit,
    Predict
}

struct CovAnalyzer <'a>{
    cov_weight_path: &'a str,
    cov_csv_path: &'a str,
    weight_set: Option<WeightSet>,
    mode: CovAnalysisMode
}

impl <'b> CovAnalyzer <'b>{
    pub fn new <'a>(cov_weight_path: &'a str, cov_csv_path: &'a str, mode: CovAnalysisMode) -> CovAnalyzer<'a>{
        unimplemented!()
    }

}

impl <'a> RunCov for CovAnalyzer<'a> {
    fn run_cov_pipeline(score_frame: &DataFrame, score_names: &[String],) -> Result<DataFrame>{
        unimplemented!()
    }
}

