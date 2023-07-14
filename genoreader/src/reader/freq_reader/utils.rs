use polars::series::Series;
use polars::prelude::{DataFrame, ChunkedArray, BooleanType, NamedFrom};
use anyhow::Result;

pub fn create_mask_u32(
    v: &[u32],
    inv: bool,
    data_frame: &DataFrame,
) -> Result<ChunkedArray<BooleanType>> {
    let v: Vec<u32> = v.iter().map(|&e| e as u32).collect();
    let v: Series = Series::new("whatever", v);
    let mask = data_frame.column("IDX")?;
    let mask = match inv {
        false => mask.is_in(&v)?,
        true => !mask.is_in(&v)?,
    };

    Ok(mask)
}
