pub mod beta;
pub mod reader;

pub use beta::Beta;
pub use reader::{read_bed::BedReader, ReadGenotype};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_predict() {
        let beta_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/model_demo/Weights.tsv";
        let beta = Beta::new(beta_path).unwrap();
        dbg!("{}", beta.beta);

        let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG.bed";
        let mut bed = BedReader::new(bed_path).unwrap();
    }
}
