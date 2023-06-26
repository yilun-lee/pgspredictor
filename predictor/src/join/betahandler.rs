use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{anyhow, Result};
use polars::{
    lazy::dsl::{all_exprs, col, lit, when},
    prelude::{DataFrame, IntoLazy},
};

use crate::meta::QrangeOrScorenames;

/// preprocess beta by select import cols, filter NaN and expand score column
/// according to q-ranges.
pub fn handle_beta(
    mut beta: DataFrame,
    q_range: &QrangeOrScorenames,
    my_cols: &Vec<String>,
) -> Result<DataFrame> {
    // filter beta
    beta = beta
        .select(my_cols)?
        .lazy()
        .filter(all_exprs([col("*").is_not_null()]))
        .collect()?;

    // get new beta from q range and get new score_names
    beta = match q_range {
        QrangeOrScorenames::QRange(v) => v.expand_beta(beta)?,
        QrangeOrScorenames::ScoreNameRaws(_) => beta,
    };
    Ok(beta)
}

#[derive(Clone, Debug)]
pub struct QRange<'a> {
    name: Vec<String>,
    from: Vec<f32>,
    to: Vec<f32>,
    pub score_names: Vec<String>,
    pub score_names_raw: &'a Vec<String>,
}

/// pub fun
impl<'a> QRange<'_> {
    pub fn new<'b>(q_ranges_path: &'b str, score_names: &'b Vec<String>) -> Result<QRange<'b>> {
        // create empty
        let mut q_range: QRange = QRange::new_empty(score_names);
        // read
        let file = File::open(q_ranges_path)?;
        for line in BufReader::new(file).lines() {
            let line = line?.replace(['\n', '\r'], "");
            q_range.add_one_line(&line)?;
        }
        // get score_names
        for score_name in score_names {
            q_range.score_names.push(score_name.to_owned());
            for name in &q_range.name {
                let new_name = format!("{score_name}_{name}");
                q_range.score_names.push(new_name);
            }
        }
        Ok(q_range)
    }

    pub fn expand_beta(&self, mut beta: DataFrame) -> Result<DataFrame> {
        let mut name: &str;
        let mut from: f32;
        let mut to: f32;
        // loop by qrange
        for i in 0..self.len() {
            (name, from, to) = self.get(i)?;
            // loop by score name, add new score one by one
            for score_name in self.score_names_raw {
                let new_name = format!("{score_name}_{name}");
                beta = beta
                    .lazy()
                    .with_columns([when(col("P").gt_eq(lit(from)).and(col("P").lt(lit(to))))
                        .then(col(score_name))
                        .otherwise(lit(0.))
                        .alias(&new_name)])
                    .collect()?;
            }
        }
        Ok(beta)
    }
}

/// private
impl<'a> QRange<'a> {
    fn new_empty(score_names: &Vec<String>) -> QRange {
        QRange {
            name: vec![],
            from: vec![],
            to: vec![],
            score_names: vec![],
            score_names_raw: score_names,
        }
    }

    fn add_one_line(&mut self, line: &str) -> Result<()> {
        for (cc, i) in (0_u32..).zip(line.split('\t')) {
            match cc {
                0 => self.name.push(i.to_string()),
                1 => self.from.push(i.parse::<f32>()?),
                2 => self.to.push(i.parse::<f32>()?),
                _ => {
                    return Err(anyhow!(
                        "There are more then three columns in q-ranges file, or the deliminator \
                         is not tab "
                    ))
                }
            }
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.name.len()
    }

    fn get(&self, idx: usize) -> Result<(&str, f32, f32)> {
        let ll = self.len();
        if idx >= ll {
            return Err(anyhow!(
                "Get {idx} qrange object out of index, which has {ll} items"
            ));
        }
        Ok((&self.name[idx], self.from[idx], self.to[idx]))
    }
}
