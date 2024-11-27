use std::{fmt::Display, iter::Sum, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SNAFU(i64);

impl From<&SNAFU> for i64 {
    fn from(value: &SNAFU) -> Self {
        value.0
    }
}

impl From<SNAFU> for i64 {
    fn from(value: SNAFU) -> Self {
        value.0
    }
}

impl Sum for SNAFU {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sum = iter.map(|s| s.0).sum();
        Self(sum)
    }
}

impl FromStr for SNAFU {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = 0;
        for b in s.bytes() {
            value *= 5;
            let add_val = match b {
                b'0'..=b'2' => (b - b'0') as i64,
                b'-' => -1,
                b'=' => -2,
                _ => return Err(anyhow::anyhow!("Unexpected SNAFU digit: {}", b)),
            };
            value += add_val;
        }
        Ok(Self(value))
    }
}

impl Display for SNAFU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: This can be a fixed size array to avoid heap allocations
        let mut buf = vec![];
        const RADIX: u64 = 5;

        let mut num = self.0.unsigned_abs();
        loop {
            let digit = (num % RADIX) as u8;
            num /= RADIX;

            let (digit, rem) = match digit {
                0..=2 => (b'0' + digit, 0),
                3 => (b'=', 1),
                4 => (b'-', 1),
                _ => unreachable!(),
            };
            buf.push(digit);
            num += rem;

            if num == 0 {
                break;
            }
        }
        // TODO: We could maybe avoid allocating strings
        buf.reverse();
        let s = String::from_utf8_lossy(&buf);
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest]
    #[case("1=", 3)]
    #[case("1=-1=", 353)]
    #[case("1=-0-2", 1747)]
    #[case("12111", 906)]
    fn parse_snafu(#[case] s: &str, #[case] expected: i64) {
        let val = SNAFU::from_str(s).unwrap();
        assert_eq!(val.0, expected);
    }

    #[rstest]
    #[case(3, "1=")]
    #[case(4890, "2=-1=0")]
    fn displays_correctly(#[case] snafu: i64, #[case] expected: &str) {
        let val = SNAFU(snafu).to_string();
        assert_eq!(val, expected);
    }
}
