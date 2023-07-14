const MASK55: u8 = 0b01010101;

fn nyp_lower_digit_u8(x: u8) -> u8 {
    x & MASK55
}

fn nyp_higher_digit_u8(x: u8) -> u8 {
    (x >> 1) & MASK55
}

pub fn nonmissing_mask_u8(x: u8) -> u8 {
    let x10 = nyp_higher_digit_u8(x) | nyp_lower_digit_u8(!x);
    return x10 | (x10 << 1);
}
//

mod tests {
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
