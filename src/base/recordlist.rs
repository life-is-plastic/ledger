use crate::base;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recordlist(Vec<base::Record>);

impl Recordlist {
    pub fn new() -> Self {
        Self::default()
    }

    fn from_vec(mut inner: Vec<base::Record>) -> Self {
        inner.sort_by_key(base::Record::date);
        Self(inner)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn spanned_interval(&self) -> base::Interval {
        let start = match self.0.first() {
            Some(r) => r.date(),
            None => return base::Interval::EMPTY,
        };
        let end = match self.0.last() {
            Some(r) => r.date(),
            None => unreachable!(),
        };
        base::Interval { start, end }
    }

    pub fn slice_spanning_interval(&self, interval: base::Interval) -> &[base::Record] {
        if interval.is_empty() {
            return &[];
        }
        let i = self.0.partition_point(|r| r.date() < interval.start);
        let j = i + self.0[i..].partition_point(|r| r.date() <= interval.end);
        &self.0[i..j]
    }

    pub fn insert(&mut self, r: base::Record) {
        let i = self.0.partition_point(|x| x.date() <= r.date());
        self.0.insert(i, r);
    }

    fn index_of(&self, dt: base::Date, iid: usize) -> Option<usize> {
        let i = self.0.partition_point(|x| x.date() < dt);
        let j = i + self.0[i..].partition_point(|x| x.date() <= dt);
        let k = i.checked_add(iid)?;
        match k.cmp(&j) {
            std::cmp::Ordering::Less => Some(k),
            _ => None,
        }
    }

    /// Returns the record at the given date and index-in-date, or `None` if
    /// input is out of bounds.
    pub fn get(&self, dt: base::Date, iid: usize) -> Option<&base::Record> {
        let i = self.index_of(dt, iid)?;
        Some(&self.0[i])
    }

    /// Removes and returns the record at the given date and index-in-date. If
    /// input is out of bounds, returns `None` and leaves record list
    /// unmodified.
    pub fn remove(&mut self, dt: base::Date, iid: usize) -> Option<base::Record> {
        let i = self.index_of(dt, iid)?;
        Some(self.0.remove(i))
    }

    pub fn iter(&self) -> impl Iterator<Item = &base::Record> {
        self.0.iter()
    }

    pub fn iter_with_iid(&self) -> impl Iterator<Item = (usize, &base::Record)> {
        let mut iid = 0;
        self.iter().enumerate().map(move |(i, r)| {
            if i > 0 && r.date() > self.0[i - 1].date() {
                iid = 0;
            }
            let ret = (iid, r);
            iid += 1;
            ret
        })
    }
}

impl IntoIterator for Recordlist {
    type Item = base::Record;
    type IntoIter = std::vec::IntoIter<base::Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<base::Record> for Recordlist {
    fn from_iter<T: IntoIterator<Item = base::Record>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a base::Record> for Recordlist {
    fn from_iter<T: IntoIterator<Item = &'a base::Record>>(iter: T) -> Self {
        iter.into_iter().cloned().collect()
    }
}

impl std::fmt::Display for Recordlist {
    /// Writes a terminating newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in self.iter() {
            writeln!(f, "{}", r)?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid record at line {line}")]
pub struct ParseError {
    line: usize,
    source: serde_json::Error,
}

impl std::str::FromStr for Recordlist {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .map(str::trim)
            .enumerate()
            .filter(|(_, x)| !x.is_empty())
            .map(|(i, x)| {
                x.parse::<base::Record>().map_err(|e| ParseError {
                    line: i + 1,
                    source: e,
                })
            })
            .collect::<Result<Self, _>>()
    }
}

impl TryFrom<&str> for Recordlist {
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
    fn test_sort_on_construction() {
        let rl = Recordlist::from_vec(vec![
            r#"{"d":"2015-03-30","c":"aaa","a":999}"#.parse::<base::Record>().unwrap(),
            r#"{"d":"2014-03-30","c":"bbb","a":888}"#.parse::<base::Record>().unwrap(),
            r#"{"d":"2016-03-30","c":"ccc","a":777}"#.parse::<base::Record>().unwrap(),
            r#"{"d":"2013-03-30","c":"ddd","a":666}"#.parse::<base::Record>().unwrap(),
        ]);
        let want_inner = vec![
            r#"{"d":"2013-03-30","c":"ddd","a":666}"#.parse::<base::Record>().unwrap(),
            r#"{"d":"2014-03-30","c":"bbb","a":888}"#.parse::<base::Record>().unwrap(),
            r#"{"d":"2015-03-30","c":"aaa","a":999}"#.parse::<base::Record>().unwrap(),
            r#"{"d":"2016-03-30","c":"ccc","a":777}"#.parse::<base::Record>().unwrap(),
        ];
        assert_eq!(rl.0, want_inner)
    }

    #[rstest]
    #[case("[]", "invalid record at line 1")]
    #[case(
        r#"
            []
        "#,
        "invalid record at line 2"
    )]
    #[case(
        r#"

            {"d":"2015-03-30","c":"","a":111}
            {"d":"2015-03-30","c":"","a":111}
        "#,
        "invalid record at line 3"
    )]
    fn test_fromstr_errormsg(#[case] s: &str, #[case] want: &str) {
        assert_eq!(s.parse::<Recordlist>().unwrap_err().to_string(), want)
    }

    #[rstest]
    #[case("", base::Interval::EMPTY)]
    #[case(r#"{"d":"2015-03-30","c":"abc","a":111}"#, "2015-03-30")]
    #[case(
        r#"
            {"d":"0000-01-31","c":"aaa","a":0}
            {"d":"2015-03-30","c":"b","a":1}
            {"d":"2015-03-30","c":"bb","a":1}
            {"d":"2015-03-31","c":"ccc","a":123456}
            {"d":"2015-04-01","c":"ddd","a":123456}
            {"d":"2015-04-02","c":"ddd","a":123456}
        "#,
        "0000-01-31:2015-04-02"
    )]
    fn test_spanned_interval(#[case] rl: Recordlist, #[case] want: base::Interval) {
        assert_eq!(rl.spanned_interval(), want)
    }

    #[rstest]
    #[case("", base::Interval::EMPTY, "")]
    #[case(r#"{"d":"2015-03-30","c":"abc","a":111}"#, base::Interval::EMPTY, "")]
    #[case(r#"{"d":"2015-03-30","c":"abc","a":111}"#, "2000-01-01", "")]
    #[case(
        r#"
            {"d":"0000-01-31","c":"aaa","a":0}
            {"d":"2015-03-30","c":"b","a":1}
            {"d":"2015-03-30","c":"bb","a":1}
            {"d":"2015-03-31","c":"ccc","a":123456}
            {"d":"2015-04-01","c":"ddd","a":123456}
            {"d":"2015-04-02","c":"ddd","a":123456}
        "#,
        ":",
        r#"
            {"d":"0000-01-31","c":"aaa","a":0}
            {"d":"2015-03-30","c":"b","a":1}
            {"d":"2015-03-30","c":"bb","a":1}
            {"d":"2015-03-31","c":"ccc","a":123456}
            {"d":"2015-04-01","c":"ddd","a":123456}
            {"d":"2015-04-02","c":"ddd","a":123456}
        "#
    )]
    #[case(
        r#"
            {"d":"0000-01-31","c":"aaa","a":0}
            {"d":"2015-03-30","c":"b","a":1}
            {"d":"2015-03-30","c":"bb","a":1}
            {"d":"2015-03-31","c":"ccc","a":123456}
            {"d":"2016-04-01","c":"ddd","a":123456}
            {"d":"2017-04-02","c":"ddd","a":123456}
        "#,
        "2014-01-01:2017-01-01",
        r#"
            {"d":"2015-03-30","c":"b","a":1}
            {"d":"2015-03-30","c":"bb","a":1}
            {"d":"2015-03-31","c":"ccc","a":123456}
            {"d":"2016-04-01","c":"ddd","a":123456}
        "#
    )]
    fn test_slice_spanning_interval(
        #[case] rl: Recordlist,
        #[case] interval: base::Interval,
        #[case] want: Recordlist,
    ) {
        assert_eq!(
            rl.slice_spanning_interval(interval),
            want.slice_spanning_interval(base::Interval::MAX)
        )
    }

    #[rstest]
    #[case(
        "",
        r#"{"d":"2015-03-30","c":"abc","a":111}"#,
        r#"{"d":"2015-03-30","c":"abc","a":111}"#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        r#"{"d":"2015-03-01","c":"abc","a":111}"#,
        r#"
            {"d":"2015-03-01","c":"abc","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        r#"{"d":"2015-03-30","c":"abc","a":111}"#,
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"abc","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        r#"{"d":"2015-03-31","c":"abc","a":111}"#,
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-31","c":"abc","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        r#"{"d":"2015-04-01","c":"abc","a":111}"#,
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
            {"d":"2015-04-01","c":"abc","a":111}
        "#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        r#"{"d":"2015-05-01","c":"abc","a":111}"#,
        r#"
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
            {"d":"2015-05-01","c":"abc","a":111}
        "#
    )]
    fn test_insert(#[case] mut rl: Recordlist, #[case] r: base::Record, #[case] want: Recordlist) {
        rl.insert(r);
        assert_eq!(rl, want)
    }

    #[rstest]
    #[case("", "2015-03-30", 0, "")]
    #[case(r#"{"d":"2015-03-30","c":"category","a":111}"#, "2015-03-30", 0, "")]
    #[case(
        r#"{"d":"2015-03-30","c":"category","a":111}"#,
        "2015-03-30",
        1,
        r#"{"d":"2015-03-30","c":"category","a":111}"#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"abc","a":111}
            {"d":"2015-03-30","c":"def","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        "2015-03-30",
        0,
        r#"
            {"d":"2015-03-30","c":"def","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#
    )]
    #[case(
        r#"
            {"d":"2015-03-30","c":"abc","a":111}
            {"d":"2015-03-30","c":"def","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        "2015-03-30",
        1,
        r#"
            {"d":"2015-03-30","c":"abc","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#
    )]
    fn test_remove(
        #[case] mut rl: Recordlist,
        #[case] dt: base::Date,
        #[case] iid: usize,
        #[case] want: Recordlist,
    ) {
        let removed = rl != want;
        assert_eq!(rl.remove(dt, iid).is_some(), removed);
        assert_eq!(rl, want);
    }

    #[rstest]
    #[case("", &[])]
    #[case(
        r#"
            {"d":"2015-03-01","c":"abc","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-03-30","c":"category","a":111}
            {"d":"2015-04-01","c":"category","a":111}
        "#,
        &[0, 0, 1, 2, 0],
    )]
    fn test_iter_with_iid(#[case] rl: Recordlist, #[case] want_iids: &[usize]) {
        let got = rl.iter_with_iid().map(|(i, _)| i).collect::<Vec<_>>();
        assert_eq!(got, want_iids)
    }
}
