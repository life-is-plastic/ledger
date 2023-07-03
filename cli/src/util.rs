/// Returns a new record list such that each record:
/// - Is in 'interval'
/// - Matches any wildcard pattern in 'categories'
/// - Does not match any wildcard pattern in 'not_categories'
pub fn filter_rl<T, U>(
    rl: &lib::Recordlist,
    interval: lib::Interval,
    categories: &[T],
    not_categories: &[U],
) -> lib::Recordlist
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    let incl = categories
        .iter()
        .map(|s| wildmatch::WildMatch::new(s.as_ref()))
        .collect::<Vec<_>>();
    let excl = not_categories
        .iter()
        .map(|s| wildmatch::WildMatch::new(s.as_ref()))
        .collect::<Vec<_>>();
    rl.slice_spanning_interval(interval)
        .iter()
        .filter({
            |r| {
                incl.iter().any(|p| p.matches(r.category().as_str()))
                    && !excl.iter().any(|p| p.matches(r.category().as_str()))
            }
        })
        .collect::<lib::Recordlist>()
}

#[cfg(test)]
pub mod testing {
    pub struct Env {
        _td: tempfile::TempDir,
        pub fs: lib::Fs,
        pub stdout: Vec<u8>,
        pub config: lib::Config,
    }

    #[rstest::fixture]
    pub fn env() -> Env {
        let _td = tempfile::TempDir::new().unwrap();
        let fs = lib::Fs::new(_td.path());
        let config = lib::Config {
            first_index_in_date: 0,
            lim_account_type: None,
            unsigned_is_positive: true,
            use_colored_output: false,
            use_unicode_symbols: false,
        };
        Env {
            _td,
            fs,
            stdout: Vec::new(),
            config,
        }
    }

    /// TODO(remove)
    #[rstest::fixture]
    pub fn rl() -> lib::Recordlist {
        r#"
            { "d": "1999-02-10",    "a": 91566,     "c": "a",       "n": "" }
            { "d": "1999-02-10",    "a": -91525,    "c": "a/b/c",   "n": "" }
            { "d": "1999-02-10",    "a": 57183,     "c": "a",       "n": "" }
            { "d": "1999-02-10",    "a": -71480,    "c": "a/c/b",   "n": "eu" }
            { "d": "1999-02-10",    "a": -10917,    "c": "a/c/b",   "n": "" }
            { "d": "1999-02-10",    "a": -6723,     "c": "a/b",     "n": "" }
            { "d": "1999-02-10",    "a": 15815,     "c": "a/c",     "n": "" }
            { "d": "1999-02-10",    "a": 17142,     "c": "a/c",     "n": "" }
            { "d": "1999-02-10",    "a": 53181,     "c": "a/c",     "n": "" }
            { "d": "1999-02-10",    "a": 87040,     "c": "c",       "n": "sit" }
            { "d": "1999-02-10",    "a": -10175,    "c": "c",       "n": "" }
            { "d": "1999-02-10",    "a": 33806,     "c": "a/b",     "n": "" }
            { "d": "1999-03-16",    "a": -77884,    "c": "a/b/c",   "n": "" }
            { "d": "1999-04-20",    "a": 13903,     "c": "a/c/b",   "n": "" }
            { "d": "1999-07-04",    "a": -30505,    "c": "a/c/b",   "n": "" }
            { "d": "1999-08-07",    "a": 76908,     "c": "c",       "n": "" }
            { "d": "1999-08-12",    "a": 24893,     "c": "b/c",     "n": "" }
            { "d": "1999-09-21",    "a": 41739,     "c": "a/c/b",   "n": "ipsum" }
            { "d": "1999-12-25",    "a": 62499,     "c": "a",       "n": "" }
            { "d": "2000-01-07",    "a": -4113,     "c": "a/b/c",   "n": "" }
            { "d": "2000-01-09",    "a": -34319,    "c": "a/b",     "n": "" }
            { "d": "2000-01-10",    "a": -7924,     "c": "a/b/c",   "n": "" }
            { "d": "2000-04-29",    "a": -78377,    "c": "a/b/c",   "n": "" }
            { "d": "2000-05-04",    "a": -44616,    "c": "a",       "n": "" }
            { "d": "2000-06-06",    "a": -46086,    "c": "b/c",     "n": "" }
            { "d": "2000-07-19",    "a": 21622,     "c": "a",       "n": "" }
            { "d": "2000-07-28",    "a": -51771,    "c": "a",       "n": "lorem" }
            { "d": "2000-08-30",    "a": -13744,    "c": "a/c",     "n": "" }
            { "d": "2000-08-31",    "a": -10735,    "c": "a/b/c",   "n": "" }
            { "d": "2000-09-05",    "a": -12284,    "c": "a/b",     "n": "" }
            { "d": "2000-09-29",    "a": -28668,    "c": "a/c",     "n": "" }
            { "d": "2000-11-14",    "a": 43653,     "c": "a/c",     "n": "" }
            { "d": "2001-01-21",    "a": -67313,    "c": "a/b",     "n": "" }
            { "d": "2001-01-21",    "a": 596,       "c": "a",       "n": "" }
            { "d": "2001-01-21",    "a": 56645,     "c": "c",       "n": "" }
            { "d": "2001-03-05",    "a": 84148,     "c": "a/b/c",   "n": "" }
            { "d": "2001-06-20",    "a": 38577,     "c": "a/c",     "n": "" }
            { "d": "2001-06-25",    "a": 61267,     "c": "a",       "n": "" }
            { "d": "2001-07-06",    "a": -53324,    "c": "a/c",     "n": "" }
            { "d": "2001-07-27",    "a": -99671,    "c": "b",       "n": "" }
            { "d": "2001-09-12",    "a": -55816,    "c": "a",       "n": "" }
            { "d": "2001-10-06",    "a": 77225,     "c": "c",       "n": "" }
            { "d": "2001-11-06",    "a": -62941,    "c": "c",       "n": "" }
            { "d": "2001-11-29",    "a": -20569,    "c": "b",       "n": "" }
            { "d": "2001-12-02",    "a": 17797,     "c": "a/b",     "n": "" }
        "#
        .parse()
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use rstest::fixture;
    use rstest::rstest;

    use super::*;

    #[fixture]
    fn rl() -> lib::Recordlist {
        r#"
            {"d":"2015-03-01","c":"aaa","a":10000}
            {"d":"2015-03-30","c":"aaa","a":10000}
            {"d":"2015-03-31","c":"bbb","a":5000}
            {"d":"2015-04-15","c":"ccc","a":-2000}
            {"d":"2015-04-29","c":"aaa","a":-2000}
            {"d":"2015-05-02","c":"bbb","a":-2000}
            {"d":"2015-05-05","c":"ccc","a":2000}
            {"d":"2015-05-20","c":"aaa","a":2000}
        "#
        .parse()
        .unwrap()
    }

    #[rstest]
    #[case(lib::Interval::EMPTY, &["*"], &[], "")]
    #[case(lib::Interval::MAX, &[], &[], "")]
    #[case(lib::Interval::MAX, &["*"], &["*"], "")]
    #[case(lib::Interval::MAX, &["*"], &[], self::rl())]
    #[case(
        "2015-03-30:2015-05-10",
        &["*b*", "c*"],
        &["*a"],
        r#"
            {"d":"2015-03-31","c":"bbb","a":5000}
            {"d":"2015-04-15","c":"ccc","a":-2000}
            {"d":"2015-05-02","c":"bbb","a":-2000}
            {"d":"2015-05-05","c":"ccc","a":2000}
        "#
    )]
    fn test_filter_rl(
        rl: lib::Recordlist,
        #[case] interval: lib::Interval,
        #[case] categories: &[&str],
        #[case] not_categories: &[&str],
        #[case] want: lib::Recordlist,
    ) {
        let got = filter_rl(&rl, interval, categories, not_categories);
        assert_eq!(got, want);
    }
}
