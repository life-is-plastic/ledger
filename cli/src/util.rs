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
    use super::*;
    use rstest::fixture;
    use rstest::rstest;

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
