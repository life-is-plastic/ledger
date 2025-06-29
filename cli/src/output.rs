/// Output of a successful command invocation. This is an intermediate
/// representation of what eventually gets written to stdout. Its purpose is to
/// aid in testing; if we trust that IRs correctly transform to final outputs,
/// then comparing IRs is much easier than comparing strings that are to be sent
/// to stdout.
///
/// When working with an `Output`, use `write!` instead of `writeln!`.
/// `Output::to_string()` is guaranteed to have a terminating newline.
#[derive(Debug, PartialEq, Eq)]
pub enum Output {
    Str(String),
    TreeForSum(base::tree::forsum::Config, base::Interval),
    TreeForView(base::tree::forview::Config),
    Barchart(base::barchart::Config),
    Limitprinter(base::limitprinter::Config),
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Str(s) => {
                if s.ends_with('\n') {
                    write!(f, "{}", s)
                } else {
                    writeln!(f, "{}", s)
                }
            }
            Output::TreeForSum(config, interval) => write!(
                f,
                "{} to {}\n{}",
                interval.start,
                interval.end,
                config.to_tree(),
            ),
            Output::TreeForView(config) => {
                if config.rl.is_empty() {
                    writeln!(f, "No transactions.")
                } else {
                    write!(f, "{}", config.to_tree())
                }
            }
            Output::Barchart(config) => {
                if config.rl.is_empty() {
                    writeln!(f, "No transactions.")
                } else {
                    write!(f, "{}", config.to_barchart())
                }
            }
            Output::Limitprinter(config) => write!(f, "{}", config.to_limitprinter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use rstest::rstest;

    #[rstest]
    #[case::str_without_newline(Output::Str("asdf".into()), "asdf\n")]
    #[case::str_with_newline(Output::Str("asdf\n".into()), "asdf\n")]
    #[case::tree_for_view_empty(
        Output::TreeForView(base::tree::forview::Config {
            charset: Default::default(),
            first_iid: 0,
            rl: base::Recordlist::new(),
            leaf_string_postprocessor: None,
        }),
        "No transactions.\n",
    )]
    #[case::tree_for_view(
        Output::TreeForView(base::tree::forview::Config {
            charset: Default::default(),
            first_iid: 0,
            rl: r#"{"d":"0000-01-01","c":"abc","a":111,"n":"note"}"#.parse().unwrap(),
            leaf_string_postprocessor: None,
        }),
        indoc!("
            0000
            `-- Jan
                `-- 1st
                    `-- 0 -- 1.11  abc: note
        "),
    )]
    #[case::barchart_empty(
        Output::Barchart(base::barchart::Config {
            charset: Default::default(),
            bounds: base::Interval::MAX,
            unit: base::Datepart::Year,
            term_width: 80,
            rl: base::Recordlist::new(),
        }),
        "No transactions.\n",
    )]
    #[case::barchart(
        Output::Barchart(base::barchart::Config {
            charset: Default::default(),
            bounds: base::Interval::MAX,
            unit: base::Datepart::Year,
            term_width: 80,
            rl: r#"{"d":"0000-01-01","c":"abc","a":111,"n":"note"}"#.parse().unwrap(),
        }),
        "0000 |+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 1.11\n",
    )]
    fn test_to_string(#[case] output: Output, #[case] want: impl Into<String>) {
        assert_eq!(output.to_string(), want.into())
    }
}
