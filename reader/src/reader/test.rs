#[cfg(test)]
mod tests {
    use nd::ShapeBuilder;
    use ndarray as nd;

    use super::super::{read_bed::BedReader, read_bed_nolib::BedReaderNoLib, ReadGenotype};
    #[test]
    fn test_read_bed() {
        let bed_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG.bed";
        let mut bed = BedReader::new(bed_path).unwrap();

        let sid = Some(vec![1, 2, 3, 4, 5]);
        let iid = Some(vec![8, 9, 22, 3]);
        let arr = bed.get_geno(&sid, &iid).unwrap();
        println!("{}", arr);
        let fam = bed.get_snp(&sid, false).unwrap();
        println!("{}", fam);
        let bim = bed.get_ind(&iid, false).unwrap();
        println!("{:?}", bim);
    }

    #[test]
    fn test_bed_nolib() {
        let bfile_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG";
        let sid = Some(vec![1, 2, 3, 4, 5, 900, 1234, 943, 2222, 10]);
        let iid = Some(vec![8, 9, 22, 3, 200, 4235]);

        let mut bed_reader = BedReaderNoLib::new(bfile_path).unwrap();
        let arr = bed_reader.get_geno(&sid, &iid).unwrap();
        println!("{}", arr);
        let fam = bed_reader.get_snp(&sid, false).unwrap();
        println!("{}", fam);
        let bim = bed_reader.get_ind(&iid, false).unwrap();
        println!("{:?}", bim);
    }
}
