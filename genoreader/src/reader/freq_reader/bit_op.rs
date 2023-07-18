const MASK55: u8 = 0b01010101;

fn nyp_lower_digit_u8(x: u8) -> u8 {
    x & MASK55
}

fn nyp_higher_digit_u8(x: u8) -> u8 {
    (x >> 1) & MASK55
}

pub fn nonmissing_mask_u8(x: u8) -> u8 {
    let x10 = nyp_higher_digit_u8(x) | nyp_lower_digit_u8(!x);
    x10 | (x10 << 1)
}
//


pub fn set_up_two_bits_to_value(count_a1: bool, missing_value: f32) -> [f32; 4] {
    let homozygous_primary_allele = 0.; // Major Allele
    let heterozygous_allele = 1.;
    let homozygous_secondary_allele = 2.; // Minor Allele

    if count_a1 {
        [
            homozygous_secondary_allele, // look-up 0
            missing_value,               // look-up 1
            heterozygous_allele,         // look-up 2
            homozygous_primary_allele,   // look-up 3
        ]
    } else {
        [
            homozygous_primary_allele,   // look-up 0
            missing_value,               // look-up 1
            heterozygous_allele,         // look-up 2
            homozygous_secondary_allele, // look-up 3
        ]
    }
}


mod tests {
    #[allow(unused_imports)]
    use super::nonmissing_mask_u8;

    #[test]
    fn test_mask() {
        let aa: u8 = 0b00011011;
        let b: u8 = nonmissing_mask_u8(aa);
        println!("{:#010b}", b);
        let cc = b.count_ones();
        println!("{}", cc);
    }
}