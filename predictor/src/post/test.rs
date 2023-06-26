#[cfg(test)]
mod tests {
    use ndarray::array;
    use polars::{
        df,
        prelude::{DataFrame, NamedFrom},
    };

    use super::super::metrics::{cal_cor, pearson_cor_1d};

    #[test]
    fn test_cor() {
        let a = array![1., 3., 4., 5., 6., 7., 1.];
        let b = array![3., 4., 5., 6., 7., 1., 12.];
        let cor = pearson_cor_1d(&a, &b).unwrap();
        println!("{}", cor);
    }

    #[test]
    fn test_polars_cor() {
        let df: DataFrame = df!(
            "IID" => &["Apple", "Apple", "Pear", "Red", "Yellow", "Green"],
            "PHENO" => &[0.2,0.4,0.5,0.1,0.66,1.0],
            "score1" => &[1.23,0.4,0.,0.345,-0.3,1.1],
            "score2" => &[-0.03,0.01,0.011,-0.003,0.2,0.44],
            "score3" => &[0.2,0.4,0.5,0.1,0.66,1.0],
        )
        .unwrap();
        let score_names = vec![
            "score1".to_owned(),
            "score2".to_owned(),
            "score3".to_owned(),
        ];
        let cor = cal_cor(&df, &score_names).unwrap();
        println!("{:?}", cor);
    }
}
