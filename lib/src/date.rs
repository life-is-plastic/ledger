use crate::Datepart;

/// A date type without time or timezone information. Values are guaranteed to
/// be between `0000-01-01` and `9999-12-31`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    // We rely on `time::Date` using `yyyy-mm-dd` format for
    // serialization/deserialization.
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Date(time::Date);

impl Date {
    /// 0000-01-01
    pub const MIN: Self = Self(time::Date::__from_ordinal_date_unchecked(0, 1));

    /// 9999-12-31
    pub const MAX: Self = Self(time::Date::__from_ordinal_date_unchecked(
        9999,
        time::util::days_in_year(9999),
    ));

    pub const fn year(self) -> u16 {
        self.0.year() as u16
    }

    pub const fn month(self) -> u16 {
        self.0.month() as u16
    }

    pub const fn day(self) -> u16 {
        self.0.day() as u16
    }

    fn new(inner: time::Date) -> Option<Self> {
        let dt = Self(inner);
        if dt >= Self::MIN && dt <= Self::MAX {
            Some(dt)
        } else {
            None
        }
    }

    pub fn from_ymd(year: u16, month: u16, day: u16) -> Option<Self> {
        let year = i32::try_from(year).ok()?;
        let month = time::Month::try_from(u8::try_from(month).ok()?).ok()?;
        let day = u8::try_from(day).ok()?;
        time::Date::from_calendar_date(year, month, day)
            .ok()
            .and_then(Self::new)
    }

    fn from_ymd_unchecked(year: u16, month: u16, day: u16) -> Self {
        Self::from_ymd(year, month, day).expect("date should be valid")
    }

    /// Returns the local date.
    #[cfg(not(test))]
    pub fn today() -> Self {
        Self(
            time::OffsetDateTime::now_local()
                .expect("current datetime should be determinable")
                .date(),
        )
    }

    /// Returns the local date.
    #[cfg(test)]
    pub fn today() -> Self {
        Self::from_ymd_unchecked(2015, 3, 30)
    }

    pub fn format(
        self,
        fmt: &(impl time::formatting::Formattable + ?Sized),
    ) -> Result<String, time::error::Format> {
        self.0.format(fmt)
    }

    pub fn first_of(self, part: Datepart) -> Self {
        match part {
            Datepart::Day => self,
            Datepart::Month => Self::from_ymd_unchecked(self.year(), self.month(), 1),
            Datepart::Year => Self::from_ymd_unchecked(self.year(), 1, 1),
        }
    }

    pub fn last_of(self, part: Datepart) -> Self {
        match part {
            Datepart::Day => self,
            Datepart::Month => Self::from_ymd_unchecked(
                self.year(),
                self.month(),
                time::util::days_in_year_month(self.0.year(), self.0.month()) as u16,
            ),
            Datepart::Year => Self::from_ymd_unchecked(self.year(), 12, 31),
        }
    }

    /// Offsets the given date by the given datepart, returning `None` if the
    /// resultant date is out of bounds.
    ///
    /// When shifting by years or months, this function clamps the resultant
    /// date's day to the resultant month's last-day-of-month. For example, if
    /// the original date is a Feb 29, shifting by 1 year will yield the next
    /// year's Feb 28.
    pub fn shift(self, part: Datepart, offset: i32) -> Option<Self> {
        let (y, m) = match part {
            Datepart::Day => {
                return self
                    .0
                    .checked_add(time::Duration::days(offset as i64))
                    .and_then(Self::new)
            }
            Datepart::Year => ((self.year() as i32).checked_add(offset)?, self.0.month()),
            Datepart::Month => {
                let mut y = self.year() as i32;
                let mut m = (self.month() as i32).checked_add(offset)?;
                if m > 12 {
                    y += (m - 1) / 12;
                    m = (m - 1) % 12 + 1;
                } else if m < 1 {
                    y += (m - 12) / 12;
                    m = (m % 12 + 11) % 12 + 1;
                }
                (
                    y,
                    time::Month::try_from(u8::try_from(m).expect("m should be bounded by [1, 12]"))
                        .expect("m should be bounded by [1, 12]"),
                )
            }
        };
        let d = time::util::days_in_year_month(y, m).min(self.0.day()) as u16;
        let y = u16::try_from(y).ok()?;
        let m = m as u16;
        Self::from_ymd(y, m, d)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // This relies on the fact that `time::Date` formats as `yyyy-mm-dd`.
        self.0.fmt(f)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("input is empty")]
    Empty,
    #[error(transparent)]
    BadFormat(#[from] time::error::Parse),
    #[error("date is before {} or after {}", Date::MIN, Date::MAX)]
    OutOfRange,
    #[error("first character is not one of {{y, Y, m, M, d, D}}")]
    InvalidFirstChar,
    #[error(transparent)]
    InvalidOffset(#[from] std::num::ParseIntError),
}

impl std::str::FromStr for Date {
    type Err = ParseError;

    /// Parses a string to a date. Inputs must be in one of the following formats:
    /// - `yyyy-mm-dd`
    /// - `xn` where `x` is one of `{y, Y, m, M, d, D}` and `n` is an integer
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Self::Err::Empty);
        }
        if s.as_bytes()[0].is_ascii_digit() {
            return time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT)
                .map_err(Self::Err::BadFormat)
                .and_then(|x| Self::new(x).ok_or(Self::Err::OutOfRange));
        }

        let bytes = s.as_bytes();
        let offset: i32 = if bytes.len() == 1 {
            0
        } else {
            std::str::from_utf8(&bytes[1..])
                .map_err(|_| Self::Err::InvalidFirstChar)?
                .parse::<i32>()?
        };
        let today = Self::today();
        match bytes[0] as char {
            'd' | 'D' => today.shift(Datepart::Day, offset),
            'y' => today.first_of(Datepart::Year).shift(Datepart::Year, offset),
            'Y' => today.last_of(Datepart::Year).shift(Datepart::Year, offset),
            'm' => today
                .first_of(Datepart::Month)
                .shift(Datepart::Month, offset),
            'M' => today
                .shift(Datepart::Month, offset)
                .map(|dt| dt.last_of(Datepart::Month)),
            _ => None,
        }
        .ok_or(Self::Err::InvalidFirstChar)
    }
}

impl TryFrom<&str> for Date {
    type Error = <Self as std::str::FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_min_max_consts() {
        assert_eq!(Date::MIN, Date::from_ymd(0, 1, 1).unwrap());
        assert_eq!(Date::MAX, Date::from_ymd(9999, 12, 31).unwrap());
    }

    #[rstest]
    #[case("2015-03-30", Date::from_ymd(2015, 3, 30).unwrap())]
    #[case("0000-01-01", Date::from_ymd(0, 1, 1).unwrap())]
    #[case("9999-12-31", Date::from_ymd(9999, 12, 31).unwrap())]
    fn test_iso8601_conv(#[case] s: &str, #[case] dt: Date) {
        assert_eq!(s.parse::<Date>().unwrap(), dt);
        assert_eq!(dt.to_string(), s);
    }

    #[rstest]
    #[case("2015-03-30", "year", "2015-01-01")]
    #[case("2015-03-30", "month", "2015-03-01")]
    #[case("2015-03-30", "day", "2015-03-30")]
    fn test_first_of(#[case] dt: Date, #[case] part: Datepart, #[case] want: Date) {
        assert_eq!(dt.first_of(part), want)
    }

    #[rstest]
    #[case("2015-03-30", "year", "2015-12-31")]
    #[case("2015-03-30", "month", "2015-03-31")]
    #[case("2015-03-30", "day", "2015-03-30")]
    #[case("1700-02-15", "month", "1700-02-28")]
    #[case("1704-02-15", "month", "1704-02-29")]
    #[case("2000-02-15", "month", "2000-02-29")]
    #[case("2001-02-15", "month", "2001-02-28")]
    #[case("3000-01-15", "month", "3000-01-31")]
    #[case("3000-03-15", "month", "3000-03-31")]
    #[case("3000-04-15", "month", "3000-04-30")]
    #[case("3000-05-15", "month", "3000-05-31")]
    #[case("3000-06-15", "month", "3000-06-30")]
    #[case("3000-07-15", "month", "3000-07-31")]
    #[case("3000-08-15", "month", "3000-08-31")]
    #[case("3000-09-15", "month", "3000-09-30")]
    #[case("3000-10-15", "month", "3000-10-31")]
    #[case("3000-11-15", "month", "3000-11-30")]
    #[case("3000-12-15", "month", "3000-12-31")]
    fn test_last_of(#[case] dt: Date, #[case] part: Datepart, #[case] want: Date) {
        assert_eq!(dt.last_of(part), want)
    }

    #[rstest]
    #[case("2015-03-30", "year", 0, Date::from_ymd(2015, 3, 30))]
    #[case("2015-03-30", "year", 1, Date::from_ymd(2016, 3, 30))]
    #[case("2015-03-30", "year", -1, Date::from_ymd(2014, 3, 30))]
    #[case("2015-03-30", "year", 30, Date::from_ymd(2045, 3, 30))]
    #[case("2015-03-30", "year", i32::MAX, None)]
    #[case("2015-03-30", "month", 0, Date::from_ymd(2015, 3, 30))]
    #[case("2015-03-30", "month", 1, Date::from_ymd(2015, 4, 30))]
    #[case("2015-03-30", "month", -1, Date::from_ymd(2015, 2, 28))]
    #[case("2015-03-30", "month", 27, Date::from_ymd(2017, 6, 30))]
    #[case("2015-03-30", "month", -27, Date::from_ymd(2012, 12, 30))]
    #[case("2015-03-30", "day", 0, Date::from_ymd(2015, 3, 30))]
    #[case("2015-03-30", "day", 1, Date::from_ymd(2015, 3, 31))]
    #[case("2015-03-30", "day", -1, Date::from_ymd(2015, 3, 29))]
    #[case("2015-03-30", "day", 100, Date::from_ymd(2015, 7, 8))]
    #[case("2015-03-30", "day", -100, Date::from_ymd(2014, 12, 20))]
    #[case("0000-01-01", "day", -1, None)]
    #[case("0002-01-01", "month", -27, None)]
    #[case("0002-01-01", "year", -4, None)]
    #[case("9999-12-31", "day", 1, None)]
    fn test_shift(
        #[case] dt: Date,
        #[case] part: Datepart,
        #[case] offset: i32,
        #[case] want: Option<Date>,
    ) {
        assert_eq!(dt.shift(part, offset), want)
    }

    #[rstest]
    #[case("2015-03-30", Date::from_ymd(2015, 3, 30))]
    #[case("y", Date::today().first_of(Datepart::Year).into())]
    #[case("Y", Date::today().last_of(Datepart::Year).into())]
    #[case("y+0", Date::today().first_of(Datepart::Year).into())]
    #[case("y-0", Date::today().first_of(Datepart::Year).into())]
    #[case("y100", Date::today().shift(Datepart::Year, 100).map(|dt| dt.first_of(Datepart::Year)))]
    #[case("Y-100", Date::today().shift(Datepart::Year, -100).map(|dt| dt.last_of(Datepart::Year)))]
    #[case("m", Date::today().first_of(Datepart::Month).into())]
    #[case("M", Date::today().last_of(Datepart::Month).into())]
    #[case("m100", Date::today().shift(Datepart::Month, 100).map(|dt| dt.first_of(Datepart::Month)))]
    #[case("M-100", Date::today().shift(Datepart::Month, -100).map(|dt| dt.last_of(Datepart::Month)))]
    #[case("d", Date::today().first_of(Datepart::Day).into())]
    #[case("D", Date::today().last_of(Datepart::Day).into())]
    #[case("d100", Date::today().shift(Datepart::Day, 100))]
    #[case("D-100", Date::today().shift(Datepart::Day, -100))]
    #[case("", None)]
    #[case("0000-00-01", None)]
    #[case("0000-00-01", None)]
    #[case("10000-01-01", None)]
    #[case("015-03-30", None)]
    #[case("2015-3-30", None)]
    #[case("2015-03-3", None)]
    #[case("y+9999", None)]
    #[case("yy", None)]
    #[case("a", None)]
    #[case("a123", None)]
    #[case("\u{251c}123", None)]
    fn test_from_str(#[case] s: &str, #[case] want: Option<Date>) {
        assert_eq!(s.parse::<Date>().ok(), want)
    }
}
