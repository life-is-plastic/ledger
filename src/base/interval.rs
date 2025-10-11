use crate::base;

/// Interval defined by the inclusive bound of two dates. If `start` is greater
/// than `end`, the interval is considered empty. ALl empty intervals are
/// equivalent.
#[derive(Debug, Clone, Copy, Eq)]
pub struct Interval {
    pub start: base::Date,
    pub end: base::Date,
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.is_empty() && other.is_empty() || self.start == other.start && self.end == other.end
    }
}

impl std::hash::Hash for Interval {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let interval = if self.is_empty() { Self::EMPTY } else { *self };
        interval.start.hash(state);
        interval.end.hash(state);
    }
}

impl Interval {
    /// The largest possible interval.
    pub const MAX: Self = Self {
        start: base::Date::MIN,
        end: base::Date::MAX,
    };

    pub const EMPTY: Self = Self {
        start: base::Date::MAX,
        end: base::Date::MIN,
    };

    pub fn is_empty(self) -> bool {
        self.start > self.end
    }

    pub fn intersection(self, other: Self) -> Self {
        Self {
            start: self.start.max(other.start),
            end: self.end.min(other.end),
        }
    }

    /// Returns an iterator over subintervals.
    ///
    /// Subintervals try to span the beginning to the end of calendar
    /// years/months. For example, iterating by year over \[2000-04-15,
    /// 2003-08-10] will yield \[2000-04-15, 2000-12-31], \[2001-01-01,
    /// 2001-12-31], etc.
    pub fn iter(self, part: base::Datepart) -> impl Iterator<Item = Self> {
        struct Iter {
            bounds: Interval,
            part: base::Datepart,
            next: Option<Interval>,
        }

        impl Iterator for Iter {
            type Item = Interval;

            fn next(&mut self) -> Option<Self::Item> {
                let ret = self.next;
                if let Some(i) = self.next {
                    self.next = i.start.shift(self.part, 1).and_then(|dt| {
                        let start = dt.first_of(self.part);
                        let end = dt.last_of(self.part).min(self.bounds.end);
                        if start <= end {
                            Some(Interval { start, end })
                        } else {
                            None
                        }
                    })
                }
                ret
            }
        }

        Iter {
            bounds: self,
            part,
            next: match self.is_empty() {
                true => None,
                false => Some(Self {
                    start: self.start,
                    end: self.start.last_of(part).min(self.end),
                }),
            },
        }
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Date(#[from] base::date::ParseError),
    #[error("invalid left side")]
    Left(#[source] base::date::ParseError),
    #[error("invalid right side")]
    Right(#[source] base::date::ParseError),
}

impl std::str::FromStr for Interval {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = match s.split_once(':') {
            Some((left, right)) => (
                if left.is_empty() {
                    base::Date::MIN
                } else {
                    left.parse::<base::Date>().map_err(Self::Err::Left)?
                },
                if right.is_empty() {
                    base::Date::MAX
                } else {
                    right.parse::<base::Date>().map_err(Self::Err::Right)?
                },
            ),
            None => {
                let dt = s.parse::<base::Date>()?;
                let part = match s.as_bytes()[0] as char {
                    'y' | 'Y' => base::Datepart::Year,
                    'm' | 'M' => base::Datepart::Month,
                    _ => base::Datepart::Day,
                };
                (dt.first_of(part), dt.last_of(part))
            }
        };
        Ok(Self { start, end })
    }
}

impl TryFrom<&str> for Interval {
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
    #[case("2015-03-30:2015-03-30", "2015-03-30", "2015-03-30")]
    #[case("2015-03-30:2020-03-30", "2015-03-30", "2020-03-30")]
    #[case("2015-03-30", "2015-03-30", "2015-03-30")]
    #[case("Y:m-1", "2015-12-31", "2015-02-01")]
    #[case("y-4:3000-01-01", "2011-01-01", "3000-01-01")]
    #[case("3000-01-01:y-4", "3000-01-01", "2011-01-01")]
    #[case(":d4", "0000-01-01", "2015-04-03")]
    #[case(":", "0000-01-01", "9999-12-31")]
    #[case("D-10:", "2015-03-20", "9999-12-31")]
    fn test_from_str(#[case] s: &str, #[case] start: base::Date, #[case] end: base::Date) {
        assert_eq!(s.parse::<Interval>().unwrap(), Interval { start, end })
    }

    #[rstest]
    #[case("")]
    #[case(":a")]
    #[case("a")]
    #[case("a:d")]
    #[case("d10000000000000000000000000000000000000000000000000000000000000")]
    #[case("12345-01-01")]
    #[case("12345-01-01:")]
    fn test_from_str_failing(#[case] s: &str) {
        assert!(s.parse::<Interval>().is_err())
    }

    #[rstest]
    #[case(":", "M:m", Interval::EMPTY)]
    #[case("m6:M7", "m-4:M-3", Interval::EMPTY)]
    #[case("m-1:M+10", "m-4:M+7", "m-1:M+7")]
    #[case("d-1:d", "d:d1", "d")]
    fn test_intersection(#[case] x: Interval, #[case] y: Interval, #[case] want: Interval) {
        let got = x.intersection(y);
        assert_eq!(got, want);
    }

    #[rstest]
    #[case("d1:d-1", base::Datepart::Day, &[])]
    #[case("2015-03-30", base::Datepart::Year, &[
        "2015-03-30:2015-03-30".parse().unwrap(),
    ])]
    #[case("2015-03-30:2017-06-29", base::Datepart::Year, &[
        "2015-03-30:2015-12-31".parse().unwrap(),
        "2016-01-01:2016-12-31".parse().unwrap(),
        "2017-01-01:2017-06-29".parse().unwrap(),
    ])]
    #[case("2015-03-30", base::Datepart::Month, &[
        "2015-03-30:2015-03-30".parse().unwrap(),
    ])]
    #[case("2015-03-30:2015-05-29", base::Datepart::Month, &[
        "2015-03-30:2015-03-31".parse().unwrap(),
        "2015-04-01:2015-04-30".parse().unwrap(),
        "2015-05-01:2015-05-29".parse().unwrap(),
    ])]
    #[case("2015-03-30", base::Datepart::Day, &[
        "2015-03-30:2015-03-30".parse().unwrap(),
    ])]
    #[case("2015-03-30:2015-04-01", base::Datepart::Day, &[
        "2015-03-30:2015-03-30".parse().unwrap(),
        "2015-03-31:2015-03-31".parse().unwrap(),
        "2015-04-01:2015-04-01".parse().unwrap(),
    ])]
    fn test_iter(
        #[case] bounds: Interval,
        #[case] part: base::Datepart,
        #[case] want: &[Interval],
    ) {
        let got = bounds.iter(part).collect::<Vec<_>>();
        assert_eq!(got.as_slice(), want)
    }
}
