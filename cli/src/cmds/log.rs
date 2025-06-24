use anyhow::Context;

use crate::output::Output;
use crate::util;

/// Log a transaction
#[derive(clap::Parser)]
pub struct Log {
    /// Transaction category, case-sensitive
    ///
    /// Use '/' to indicate hierarchy. For example, in 'commute/car/gas',
    /// 'commute' is the top level category, 'commute/car' is the second level,
    /// and 'commute/car/gas' is the leaf.
    category: base::Category,

    /// Transaction amount
    ///
    /// Note that although the value 0 is allowed, such transactions are
    /// effectively ignored by commands 'sum', 'plot', and 'lim'.
    #[arg(allow_negative_numbers = true)]
    amount: CentsArg,

    /// Transaction date
    #[arg(default_value = "d")]
    date: base::Date,

    /// Optional comments about transaction
    #[arg(short, long, default_value_t, hide_default_value = true)]
    note: String,

    /// Allow logging the entry if its category does not already exist
    #[arg(short, long)]
    create: bool,
}

impl Log {
    pub fn run(
        self,
        mut rl: base::Recordlist,
        config: &base::Config,
        fs: &base::Fs,
    ) -> anyhow::Result<Output> {
        if !self.create && !rl.iter().any(|r| r.category() == &self.category) {
            anyhow::bail!("nonexistent category")
        }

        let r = base::Record::new(
            self.date,
            self.category,
            self.amount.into_cents(config.unsigned_is_negative),
            self.note,
        );
        rl.insert(r);
        fs.write(&rl).with_context(|| {
            format!(
                "failed to write '{}'",
                fs.path::<base::Recordlist>().display()
            )
        })?;
        let rl = rl
            .slice_spanning_interval(base::Interval {
                start: self.date,
                end: self.date,
            })
            .iter()
            .collect::<base::Recordlist>();
        let tr_config = base::tree::forview::Config {
            charset: util::charset_from_config(config),
            first_iid: config.first_index_in_date,
            leaf_string_postprocessor: None,
            rl,
        };
        Ok(Output::TreeForView(tr_config))
    }
}

#[derive(Clone)]
enum CentsArg {
    Signed(base::Cents),
    Unsigned(base::Cents),
}

impl CentsArg {
    fn into_cents(self, unsigned_is_negative: bool) -> base::Cents {
        match self {
            CentsArg::Signed(x) => x,
            CentsArg::Unsigned(x) => {
                if unsigned_is_negative {
                    -x.abs()
                } else {
                    x.abs()
                }
            }
        }
    }
}

impl std::str::FromStr for CentsArg {
    type Err = <base::Cents as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cents = base::Cents::from_str(s)?;
        let signed = [b'+', b'-'].contains(&s.as_bytes()[0]);
        if signed {
            Ok(Self::Signed(cents))
        } else {
            Ok(Self::Unsigned(cents))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use rstest::rstest;

    #[rstest]
    #[case(CentsArg::Signed(base::Cents(123)), false, base::Cents(123))]
    #[case(CentsArg::Signed(base::Cents(123)), true, base::Cents(123))]
    #[case(CentsArg::Signed(base::Cents(-123)), false, base::Cents(-123))]
    #[case(CentsArg::Signed(base::Cents(-123)), true, base::Cents(-123))]
    #[case(CentsArg::Unsigned(base::Cents(123)), false, base::Cents(123))]
    #[case(CentsArg::Unsigned(base::Cents(123)), true, base::Cents(-123))]
    #[case(CentsArg::Unsigned(base::Cents(-123)), false, base::Cents(123))]
    #[case(CentsArg::Unsigned(base::Cents(-123)), true, base::Cents(-123))]
    fn test_centsarg_into_cents(
        #[case] arg: CentsArg,
        #[case] unsigned_is_negative: bool,
        #[case] want: base::Cents,
    ) {
        assert_eq!(arg.into_cents(unsigned_is_negative), want)
    }

    testing::generate_testcases![
        (
            nonexistent_category,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "log", "aaa", "-1.23", "2015-03-30"],
                    res: testing::ResultMatcher::ErrGlob("nonexistent category"),
                }],
                initial_state: testing::StrState::new().with_config("{}"),
            }
        ),
        (
            normal_execution,
            testing::MutCase {
                invocations: &[
                    testing::Invocation {
                        args: &[
                            "",
                            "log",
                            "aaa",
                            "-1.23",
                            "2015-03-30",
                            "--note",
                            "qwerty",
                            "--create",
                        ],
                        res: testing::ResultMatcher::OkExact(Output::TreeForView(
                            base::tree::forview::Config {
                                charset: Default::default(),
                                first_iid: 0,
                                rl: r#"{"d":"2015-03-30","c":"aaa","a":-123,"n":"qwerty"}"#
                                    .parse()
                                    .unwrap(),
                                leaf_string_postprocessor: None,
                            }
                        )),
                    },
                    testing::Invocation {
                        args: &[
                            "",
                            "log",
                            "aaa",
                            "4.56",
                            "2015-03-30",
                            "--note",
                            "qwerty",
                            "--create"
                        ],
                        res: testing::ResultMatcher::OkExact(Output::TreeForView(
                            base::tree::forview::Config {
                                charset: Default::default(),
                                first_iid: 0,
                                rl: r#"
                                    {"d":"2015-03-30","c":"aaa","a":-123,"n":"qwerty"}
                                    {"d":"2015-03-30","c":"aaa","a":456,"n":"qwerty"}
                                "#
                                .parse()
                                .unwrap(),
                                leaf_string_postprocessor: None,
                            }
                        )),
                    },
                    testing::Invocation {
                        args: &["", "log", "aaa", "789", "2015-03-30", "--note", "qwerty"],
                        res: testing::ResultMatcher::OkExact(Output::TreeForView(
                            base::tree::forview::Config {
                                charset: Default::default(),
                                first_iid: 0,
                                rl: r#"
                                    {"d":"2015-03-30","c":"aaa","a":-123,"n":"qwerty"}
                                    {"d":"2015-03-30","c":"aaa","a":456,"n":"qwerty"}
                                    {"d":"2015-03-30","c":"aaa","a":78900,"n":"qwerty"}
                                "#
                                .parse()
                                .unwrap(),
                                leaf_string_postprocessor: None,
                            }
                        )),
                    },
                ],
                initial_state: testing::StrState::new().with_config("{}"),
                final_state: testing::State::new()
                    .with_config(base::Config::default())
                    .with_rl(
                        r#"
                            {"d":"2015-03-30","c":"aaa","a":-123,"n":"qwerty"}
                            {"d":"2015-03-30","c":"aaa","a":456,"n":"qwerty"}
                            {"d":"2015-03-30","c":"aaa","a":78900,"n":"qwerty"}
                        "#
                    ),
            }
        ),
        (
            unsigned_positive,
            testing::MutCase {
                invocations: &[testing::Invocation {
                    args: &["", "log", "aaa", "1.23", "--create"],
                    res: testing::ResultMatcher::OkExact(Output::TreeForView(
                        base::tree::forview::Config {
                            charset: Default::default(),
                            first_iid: 0,
                            rl: format!(r#"{{"d":"{}","c":"aaa","a":123}}"#, base::Date::today())
                                .parse()
                                .unwrap(),
                            leaf_string_postprocessor: None,
                        }
                    )),
                }],
                initial_state: testing::StrState::new()
                    .with_config(r#"{"unsignedIsNegative":false}"#),
                final_state: testing::State::new()
                    .with_config(r#"{"unsignedIsNegative":false}"#)
                    .with_rl(
                        format!(r#"{{"d":"{}","c":"aaa","a":123}}"#, base::Date::today()).as_str()
                    ),
            }
        ),
        (
            unsigned_negative,
            testing::MutCase {
                invocations: &[testing::Invocation {
                    args: &["", "log", "aaa", "1.23", "--create"],
                    res: testing::ResultMatcher::OkExact(Output::TreeForView(
                        base::tree::forview::Config {
                            charset: Default::default(),
                            first_iid: 0,
                            rl: format!(r#"{{"d":"{}","c":"aaa","a":-123}}"#, base::Date::today())
                                .parse()
                                .unwrap(),
                            leaf_string_postprocessor: None,
                        }
                    )),
                }],
                initial_state: testing::StrState::new()
                    .with_config(r#"{"unsignedIsNegative":true}"#),
                final_state: testing::State::new()
                    .with_config(r#"{"unsignedIsNegative":true}"#)
                    .with_rl(
                        format!(r#"{{"d":"{}","c":"aaa","a":-123}}"#, base::Date::today()).as_str()
                    ),
            }
        ),
    ];
}
