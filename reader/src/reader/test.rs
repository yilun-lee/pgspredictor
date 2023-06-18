#[cfg(test)]
mod tests {
    use nd::s;
    use ndarray as nd;

    use super::super::{read_bed_nolib::BedReaderNoLib, ReadGenotype};

    /*
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
    */

    #[test]
    fn test_nan() {
        let a = f32::NAN * 1.;
        println!("{}", a);
    }

    #[test]
    fn test_bed_nolib() {
        let bfile_path = "/Users/sox/Desktop/AILAB_DATA/Data/DEMO/DEMO_REG/DEMO_REG";
        let sid = Some(vec![1, 2, 3, 4, 5, 900, 1234, 943, 2222, 10, 10, 9999]);
        let iid = Some(vec![
            8, 9, 22, 3, 200, 4235, 1, 2, 3, 4, 5, 900, 1234, 943, 2222, 10,
        ]);

        let bed_reader = BedReaderNoLib::new(bfile_path).unwrap();
        let mut arr = bed_reader.get_geno(&sid, &iid).unwrap();
        let fam = bed_reader.get_snp(&sid, false).unwrap();
        println!("{}", fam);
        let bim = bed_reader.get_ind(&iid, false).unwrap();
        println!("{:?}", bim);

        println!("{}", &arr);
        for i in 0..arr.shape()[1] {
            arr.slice_mut(s![.., i]).mapv_inplace(|x: f32| {
                if x.is_nan() {
                    return 100.;
                }
                return x;
            });
        }
        println!("{}", arr);
    }
}
