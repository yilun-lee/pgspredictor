use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use genoreader::{
    reader::{read_bed_nolib::BedReaderNoLib, ReadGenotype}, FreqBedReader, BfileSet,
};

pub fn my_bed_reader(c: &mut Criterion) {
    //let bed_path = "/Users/sox/CODE/prs-predict/data/input/test.bed";
    let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/CLU";
    
    let bfile_set = Arc::new(BfileSet::new(bed_path).unwrap());

    let mut freq_bed_reader = FreqBedReader::new(bfile_set).unwrap();

    let sid_idx: Vec<isize> = (0..(100 as isize)).collect();

    c.bench_function("my_bed_reader", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| freq_bed_reader.read_snp(&sid_idx, None, None))
    });
}

pub fn fastlmm_bed_reader(c: &mut Criterion) {
    //let bed_path = "/Users/sox/CODE/prs-predict/data/input/test";
    let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/CLU";
    let my_bed_reader = BedReaderNoLib::new(bed_path).unwrap();

    let sid_idx: Vec<isize> = (0..(100 as isize)).collect();
    let sid_idx = Some(sid_idx);
    c.bench_function("fastlmm_bed_reader", |b| {
        b.iter(|| my_bed_reader.get_geno(&sid_idx, &None))
    });
}
criterion_group!(benches, my_bed_reader, fastlmm_bed_reader);
criterion_main!(benches);
