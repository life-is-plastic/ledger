use crate::util;

/// Integral representation of monetary quantities up to two decimal places.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    derive_more::From,
    derive_more::Into,
    derive_more::Neg,
    derive_more::Sum,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Mul,
    derive_more::MulAssign,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Cents(pub i64);

impl Cents {
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Returns `cents.to_string().len()` without actually building a string.
    pub fn charlen(self) -> usize {
        let n = self.abs().0.max(100) as u64;
        let mut len = util::count_digits(n);
        len += (len - 3) / 3; // commas
        len += 1; // decimal point
        if self.0 < 0 {
            len += 2; // parentheses
        }
        len
    }

    /// Returns `cents.charlen()` assuming a non-negative quantity has a
    /// trailing space in its string representation. Having a trailing space
    /// means regardless of sign, the string representation has 3 characters
    /// after the decimal point, meaning right-aligning is equivalent to
    /// aligning on the decimal point.
    pub fn charlen_for_alignment(self) -> usize {
        self.charlen() + (self >= Self(0)) as usize
    }
}

impl std::fmt::Display for Cents {
    /// Formats with two decimal places and thousands separators. Negative
    /// quantities are wrapped in parentheses.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut cents = self.abs().0;
        let mut bytes = Vec::<u8>::new();
        macro_rules! pop_digit {
            () => {
                bytes.push(b'0' + (cents % 10) as u8);
                cents /= 10
            };
        }

        pop_digit!();
        pop_digit!();
        bytes.push(b'.');
        pop_digit!();
        let mut i = 1;
        while cents > 0 {
            if i % 3 == 0 {
                bytes.push(b',');
            }
            i += 1;
            pop_digit!();
        }
        bytes.reverse();
        if self.0 < 0 {
            bytes.insert(0, b'(');
            bytes.push(b')');
        }
        let s = std::str::from_utf8(&bytes).expect("all chars should be ascii");
        f.write_str(s)
    }
}

impl std::str::FromStr for Cents {
    type Err = std::num::ParseIntError;

    /// Parses a cents quantity from a human-readable string, which may contain
    /// comma thousands separators and any number of decimal places. Decimal
    /// places beyond the second are discarded.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.replace(',', "");
        if !["", "+", "-", ".", "+.", "-."].contains(&s.as_str()) {
            let mut chars = s.chars().collect::<Vec<_>>();
            chars.push('0');
            chars.push('0');
            if let Some(i) = chars.iter().copied().position(|c| c == '.') {
                chars.swap(i, i + 1);
                chars.swap(i + 1, i + 2);
                chars.truncate(i + 2);
            };
            s = chars.into_iter().collect::<String>();
        }
        s.parse::<i64>().map(Self)
    }
}

impl TryFrom<&str> for Cents {
    type Error = <Self as std::str::FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Cents(0), "0.00")]
    #[case(Cents(10), "0.10")]
    #[case(Cents(-123), "(1.23)")]
    #[case(Cents(123456789), "1,234,567.89")]
    #[case(Cents(-10), "(0.10)")]
    #[case(Cents(-123456789), "(1,234,567.89)")]
    #[case(Cents(i64::MIN + 1), "(92,233,720,368,547,758.07)")]
    fn test_to_string(#[case] cents: Cents, #[case] want: String) {
        let got = cents.to_string();
        assert_eq!(got, want);
        assert_eq!(cents.charlen(), got.len());
    }

    #[rstest]
    #[case("0", Cents(0))]
    #[case("0.", Cents(0))]
    #[case(".0", Cents(0))]
    #[case("0.0", Cents(0))]
    #[case("-0", Cents(0))]
    #[case("1", Cents(100))]
    #[case("+1.", Cents(100))]
    #[case("-.1", Cents(-10))]
    #[case("123456", Cents(12345600))]
    #[case("-123456", Cents(-12345600))]
    #[case("1234.56", Cents(123456))]
    #[case("1,234.56", Cents(123456))]
    #[case("0001,234.56789", Cents(123456))]
    #[case("-,,1,23,,4.5,,,6,7", Cents(-123456))]
    fn test_from_str(#[case] s: &str, #[case] want: Cents) {
        assert_eq!(s.parse::<Cents>().unwrap(), want)
    }

    #[rstest]
    #[case("")]
    #[case("+")]
    #[case("-")]
    #[case(".")]
    #[case("+.")]
    #[case("-.")]
    #[case("+a.")]
    #[case("+.a")]
    #[case("+-0.")]
    #[case("--0.")]
    fn test_from_str_failing(#[case] s: &str) {
        assert!(s.parse::<Cents>().is_err())
    }
}
