#[cfg(test)]
mod tests {
    use super::super::Beta;
    #[test]
    fn test_read_bed() {
        let beta_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/model_demo/Weights.tsv";
        let bed = Beta::new(beta_path).unwrap();
        dbg!("{}", bed.beta);
    }
}
