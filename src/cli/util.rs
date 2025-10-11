use crate::base;

pub fn charset_from_config(config: &base::Config) -> base::Charset {
    let mut charset = base::Charset::default();
    if config.use_unicode_symbols {
        charset = charset.with_unicode()
    }
    if config.use_colored_output {
        charset = charset.with_color()
    }
    charset
}

/// If `fullmatch` is false, ensures all categories starts with and ends with
/// `*`, except for empty categories which are left alone. If `fullmatch` is
/// true, does not modify categories.
pub fn preprocess_categories<'a>(
    categories: &'a [String],
    fullmatch: bool,
) -> std::borrow::Cow<'a, [String]> {
    if fullmatch {
        return categories.into();
    }
    categories
        .iter()
        .map(|s| {
            let mut s2 = s.clone();
            if s2.is_empty() {
                return s2;
            }
            if !s2.starts_with('*') {
                s2.insert(0, '*');
            }
            if !s2.ends_with('*') {
                s2.push('*');
            }
            s2
        })
        .collect::<Vec<_>>()
        .into()
}

/// Returns a new record list such that each record:
/// - Is in 'interval'
/// - Matches any wildcard pattern in 'categories'
/// - Does not match any wildcard pattern in 'not_categories'
pub fn filter_rl<T, U>(
    rl: &base::Recordlist,
    interval: base::Interval,
    categories: &[T],
    not_categories: &[U],
) -> base::Recordlist
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
        .collect::<base::Recordlist>()
}

#[cfg(test)]
mod tests {
    use rstest::fixture;
    use rstest::rstest;

    use super::*;

    #[fixture]
    fn rl() -> base::Recordlist {
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
    #[case(
        base::Config {
            use_colored_output: false,
            use_unicode_symbols: false,
            ..base::Config::default()
        },
        base::Charset::default(),
    )]
    #[case(
        base::Config {
            use_colored_output: true,
            use_unicode_symbols: false,
            ..base::Config::default()
        },
        base::Charset::default().with_color(),
    )]
    #[case(
        base::Config {
            use_colored_output: false,
            use_unicode_symbols: true,
            ..base::Config::default()
        },
        base::Charset::default().with_unicode(),
    )]
    #[case(
        base::Config {
            use_colored_output: true,
            use_unicode_symbols: true,
            ..base::Config::default()
        },
        base::Charset::default().with_color().with_unicode(),
    )]
    fn test_charset_from_config(#[case] config: base::Config, #[case] want: base::Charset) {
        let got = charset_from_config(&config);
        assert_eq!(got, want);
    }

    #[rstest]
    #[case(&[], /*fullmatch=*/true, &[])]
    #[case(&[], /*fullmatch=*/false, &[])]
    #[case(
        &["1".into(), "".into(), "2*".into(), "**3*3".into()],
        /*fullmatch=*/true,
        &["1", "", "2*", "**3*3"]
    )]
    #[case(
        &["1".into(), "".into(), "2*".into(), "**3*3".into()],
        /*fullmatch=*/false,
        &["*1*", "","*2*", "**3*3*"]
    )]
    fn test_preprocess_categories(
        #[case] categories: &[String],
        #[case] fullmatch: bool,
        #[case] want: &[&str],
    ) {
        let got = preprocess_categories(categories, fullmatch);
        assert_eq!(got, want);
    }

    #[rstest]
    #[case(base::Interval::EMPTY, &["*"], &[], "")]
    #[case(base::Interval::MAX, &[], &[], "")]
    #[case(base::Interval::MAX, &["*"], &["*"], "")]
    #[case(base::Interval::MAX, &["*"], &[], self::rl())]
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
        rl: base::Recordlist,
        #[case] interval: base::Interval,
        #[case] categories: &[&str],
        #[case] not_categories: &[&str],
        #[case] want: base::Recordlist,
    ) {
        let got = filter_rl(&rl, interval, categories, not_categories);
        assert_eq!(got, want);
    }
}
