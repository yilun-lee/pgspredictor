#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::sync::Arc;
    use crate::{BedReaderNoLib, ReadGenotype};
    use crate::reader::freq_reader::{BfileSet,FreqBedReader};

    #[test]
    fn my_bed_reader() {
        //let bed_path = "/Users/sox/CODE/prs-predict/data/input/test.bed";
        //let in_sid_count: usize = 8574;
        //let in_iid_count: usize = 10;
        let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/CLU";
        let bfile_set = Arc::new(BfileSet::new(bed_path).unwrap());

        let mut freq_bed_reader = FreqBedReader::new(bfile_set).unwrap();

        let sid_idx: Vec<isize> = (0..(1000 as isize)).collect();

        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(1000)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .unwrap();

        let val = freq_bed_reader.read_snp(&sid_idx, None, None).unwrap();
        println!("{:?}", val);

        if let Ok(report) = guard.report().build() {
            let file: File = File::create(
                "/Users/sox/CODE/prs-predict/data/output/my_bed_reader.flamegraph.svg",
            )
            .unwrap();
            report.flamegraph(file).unwrap();
        };
    }

    #[test]
    fn fastlmm_bed_reader() {
        //let bed_path = "/Users/sox/CODE/prs-predict/data/input/test";
        let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/CLU_DATA/CLU";
        let my_bed_reader = BedReaderNoLib::new(bed_path).unwrap();

        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(1000)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .unwrap();

        let sid_idx: Vec<isize> = (0..(1000 as isize)).collect();
        let sid_idx: Option<Vec<isize>> = Some(sid_idx);
        let val = my_bed_reader.get_geno(&sid_idx, &None).unwrap();
        println!("{:?}", val);

        if let Ok(report) = guard.report().build() {
            let file = File::create(
                "/Users/sox/CODE/prs-predict/data/output/fastlmm_bed_reader.flamegraph.svg",
            )
            .unwrap();
            report.flamegraph(file).unwrap();
        };
    }
}
