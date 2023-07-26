
use ndarray::{Array2,Array1};
use polars::prelude::DataFrame;
use anyhow::Result;



/// Struct represent Standardize weight
struct StandardizeSet {
    means: Array1<f32>,
    stds: Array1<f32>,
}

/// Struct represent LM weight
struct LinearSet {
    betas: Array1<f32>,
    bias_flag: bool,
}

/// Weight struct, including [LinearSet] and [StandardizeSet]
pub struct WeightSet {
    names: Vec<String>,
    stand_set: StandardizeSet,
    lm_set: LinearSet
}

pub struct FeatureSet{
    val: Array2<f32>,
    names: Vec<String>,
}

/// basic operation pub function
impl FeatureSet {
    pub fn from_dataframe(score_frame: &DataFrame, score_names: &[String]) -> Result<FeatureSet>{
        unimplemented!()
    }

    pub fn concat(self, feat_set: FeatureSet) -> Result<FeatureSet>{
        unimplemented!()
    }

    pub fn fit(&self, y: Array1<f32>, bias_flag: bool) -> Result<WeightSet> {
        unimplemented!()
    }

    pub fn predict(&self, weight_set: &WeightSet) -> Result<Array1<f32>> {
        unimplemented!()
    }
}

/// standardize and lm, private function
impl FeatureSet {
    fn fill_na_with_mean(&mut self, means: Array1<f32>) -> Result<()>{
        unimplemented!()
    }

    fn get_mean_std(&self) -> Result<StandardizeSet>{
        unimplemented!()
    }

    fn standardize(&mut self, stand_set: &StandardizeSet) -> Result<()> {
        unimplemented!()
    }

    fn linear_fit(&self, y: Array1<f32>, bias_flag: bool) -> Result<LinearSet> {
        unimplemented!()
    }

    fn linear_predict(&self, linear_set: &LinearSet) -> Result<Array1<f32>> {
        unimplemented!()
    }
}

/// load and save weights
impl WeightSet {

    pub fn match_names(&self, score_names: &[String]) -> bool {
        unimplemented!()
    }

    pub fn load_weights(weight_path: &str) -> Result<WeightSet>{
        unimplemented!()
    }

    pub fn save_weights(&self)-> Result<()>{
        unimplemented!()
    }
}

