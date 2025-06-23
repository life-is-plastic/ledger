pub const BOUNDING_SPACES_COUNT: usize = 2;
pub const MIN_DASHES_COUNT: usize = 2;
pub const MIN_TERM_WIDTH: usize = 60;

pub const fn count_digits(n: u64) -> usize {
    if n >= 10000000000000000000 {
        return 20;
    }
    let mut count = 1;
    let mut ceil = 10;
    while n >= ceil {
        ceil *= 10;
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 1)]
    #[case(1, 1)]
    #[case(9, 1)]
    #[case(10, 2)]
    #[case(99, 2)]
    #[case(100, 3)]
    #[case(1234, 4)]
    #[case(u64::MAX, 20)]
    #[case(u64::MAX / 10, 19)]
    fn test_count_digits(#[case] n: u64, #[case] want: usize) {
        assert_eq!(count_digits(n), want)
    }
}
