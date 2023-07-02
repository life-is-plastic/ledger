use crate::Cents;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Limits(std::collections::BTreeMap<u16, Cents>);

impl Limits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set(&mut self, year: u16, limit: Cents) {
        self.0.insert(year, limit);
    }

    pub fn remove(&mut self, year: u16) -> Option<Cents> {
        self.0.remove(&year)
    }

    /// Returns total accumulated yearly room up to and including `year`.
    pub fn inception_to_year(&self, year: u16) -> i64 {
        self.iter()
            .map(|(y, room)| match y <= year {
                true => room.0,
                false => 0,
            })
            .sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, Cents)> + '_ {
        self.0.iter().map(|(&k, &v)| (k, v))
    }
}

impl std::fmt::Display for Limits {
    /// Writes a terminating newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
        writeln!(f, "{}", s)
    }
}

impl std::str::FromStr for Limits {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(indoc!("{}\n"), vec![])]
    #[case(
        indoc!(r#"
        {
          "0": 0,
          "2": 345
        }
        "#),
        vec![
            (0, Cents(0)),
            (2, Cents(345)),
        ],
    )]
    fn test_serde(#[case] s: &str, #[case] want: Vec<(u16, Cents)>) {
        let got = s.parse::<Limits>().unwrap();
        let want = Limits(want.into_iter().collect());
        assert_eq!(got, want);
        assert_eq!(got.to_string(), s);
    }

    #[test]
    fn test_crud() {
        let mut limits = Limits::new();
        limits.set(2015, Cents(1000));
        limits.set(2016, Cents(0));
        assert_eq!(
            limits,
            Limits([(2015, Cents(1000)), (2016, Cents(0))].into())
        );
        assert_eq!(limits.inception_to_year(2014), 0);
        assert_eq!(limits.inception_to_year(2015), 1000);
        assert_eq!(limits.inception_to_year(2016), 1000);

        limits.set(2016, Cents(2000));
        limits.set(2014, Cents(3000));
        assert_eq!(
            limits,
            Limits(
                [
                    (2014, Cents(3000)),
                    (2015, Cents(1000)),
                    (2016, Cents(2000))
                ]
                .into()
            )
        );
        assert_eq!(limits.inception_to_year(2013), 0);
        assert_eq!(limits.inception_to_year(2014), 3000);
        assert_eq!(limits.inception_to_year(2015), 4000);
        assert_eq!(limits.inception_to_year(2016), 6000);
        assert_eq!(limits.inception_to_year(2017), 6000);

        limits.remove(2015);
        assert_eq!(
            limits,
            Limits([(2014, Cents(3000)), (2016, Cents(2000))].into())
        );
        assert_eq!(limits.inception_to_year(2013), 0);
        assert_eq!(limits.inception_to_year(2014), 3000);
        assert_eq!(limits.inception_to_year(2015), 3000);
        assert_eq!(limits.inception_to_year(2016), 5000);
        assert_eq!(limits.inception_to_year(2017), 5000);
    }
}
