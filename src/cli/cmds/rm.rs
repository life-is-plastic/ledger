use anyhow::Context;

use crate::base;
use crate::cli;

/// Remove a transaction
#[derive(clap::Parser)]
pub struct Rm {
    /// Transaction date
    date: base::Date,

    /// Index of transaction in DATE
    index: usize,

    /// Execute the removal instead of displaying dry run changes
    #[arg(long)]
    confirm: bool,
}

impl Rm {
    pub fn run(
        self,
        mut rl: base::Recordlist,
        config: &base::Config,
        fs: &base::Fs,
    ) -> anyhow::Result<cli::Output> {
        let iid0 = self.index.wrapping_sub(config.first_index_in_date);
        if rl.get(self.date, iid0).is_none() {
            anyhow::bail!("nonexistent transaction");
        }

        let rl_for_date = rl
            .slice_spanning_interval(base::Interval {
                start: self.date,
                end: self.date,
            })
            .iter()
            .collect::<base::Recordlist>();
        let lspp = move |config: &base::tree::forview::Config,
                         r: &base::Record,
                         iid0_arg: usize,
                         mut leaf_string: String|
              -> String {
            if r.date() == self.date && iid0_arg == iid0 {
                if self.confirm {
                    let mut msg = " <- [REMOVED]".to_string();
                    if config.charset.color {
                        msg = colored::Colorize::red(msg.as_str()).to_string();
                    }
                    leaf_string.push_str(&msg);
                } else {
                    let mut msg = " <- [WOULD BE REMOVED]".to_string();
                    if config.charset.color {
                        msg = colored::Colorize::yellow(msg.as_str()).to_string();
                    }
                    leaf_string.push_str(&msg);
                }
            }
            leaf_string
        };
        let tr_config = base::tree::forview::Config {
            charset: cli::util::charset_from_config(config),
            first_iid: config.first_index_in_date,
            rl: rl_for_date,
            leaf_string_postprocessor: Some(Box::new(lspp)),
        };

        if self.confirm {
            rl.remove(self.date, iid0)
                .expect("record should have already been verified to exist");
            fs.write(&rl).with_context(|| {
                format!(
                    "failed to write '{}'",
                    fs.path::<base::Recordlist>().display()
                )
            })?;
        }

        Ok(cli::Output::TreeForView(tr_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Equality checks on `base::tree::forview::Config` does not care about the
    /// `Some` payload of `leaf_string_postprocessor`. Rather, equality only
    /// requires either both sides to be `Some`, or both sides to be `None`.
    /// This function helps generate a dummy paylod for `Some`.
    fn dummy_lspp()
    -> Box<dyn Fn(&base::tree::forview::Config, &base::Record, usize, String) -> String> {
        fn f(_: &base::tree::forview::Config, _: &base::Record, _: usize, _: String) -> String {
            String::default()
        }
        Box::new(f)
    }

    cli::testing::generate_testcases![
        (
            nonexistent,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "rm", "0000-01-01", "0", "--confirm"],
                    res: cli::testing::ResultMatcher::ErrGlob("nonexistent transaction"),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}"),
            }
        ),
        (
            bad_index,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "rm", "0000-01-01", "4"],
                    res: cli::testing::ResultMatcher::ErrGlob("nonexistent transaction"),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"0000-01-01","c":"abc","a":111}
                        {"d":"0000-01-01","c":"def","a":111,"n":"note"}
                    "#
                ),
            }
        ),
        (
            dry_run,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "rm", "0000-01-01", "1"],
                    res: cli::testing::ResultMatcher::OkExact(cli::Output::TreeForView(
                        base::tree::forview::Config {
                            charset: Default::default(),
                            first_iid: 0,
                            rl: r#"
                                {"d":"0000-01-01","c":"abc","a":111}
                                {"d":"0000-01-01","c":"def","a":111,"n":"note"}
                            "#
                            .parse()
                            .unwrap(),
                            leaf_string_postprocessor: Some(dummy_lspp()),
                        }
                    )),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"0000-01-01","c":"abc","a":111}
                        {"d":"0000-01-01","c":"def","a":111,"n":"note"}
                    "#
                ),
            }
        ),
        (
            wet_run,
            cli::testing::MutCase {
                invocations: &[cli::testing::Invocation {
                    args: &["", "rm", "0000-01-01", "1", "--confirm"],
                    res: cli::testing::ResultMatcher::OkExact(cli::Output::TreeForView(
                        base::tree::forview::Config {
                            charset: Default::default(),
                            first_iid: 0,
                            rl: r#"
                                {"d":"0000-01-01","c":"abc","a":111}
                                {"d":"0000-01-01","c":"def","a":111,"n":"note"}
                            "#
                            .parse()
                            .unwrap(),
                            leaf_string_postprocessor: Some(dummy_lspp()),
                        }
                    )),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"0000-01-01","c":"abc","a":111}
                        {"d":"0000-01-01","c":"def","a":111,"n":"note"}
                    "#
                ),
                final_state: cli::testing::State::new()
                    .with_config(base::Config::default())
                    .with_rl(r#"{"d":"0000-01-01","c":"abc","a":111}"#),
            }
        ),
    ];

    #[rstest]
    #[case::dry_run(
        Rm {
            date: base::Date::MIN,
            index: 1,
            confirm: false,
        },
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        "def: note <- [WOULD BE REMOVED]"
    )]
    #[case::wet_run(
        Rm {
            date: base::Date::MIN,
            index: 1,
            confirm: true,
        },
        r#"
            {"d":"0000-01-01","c":"abc","a":111}
            {"d":"0000-01-01","c":"def","a":111,"n":"note"}
        "#,
        "def: note <- [REMOVED]"
    )]
    fn test_leaf_string_postprocessor(
        #[case] rm: Rm,
        #[case] rl: base::Recordlist,
        #[case] want_in_output: &str,
    ) {
        let (fs, _td) = cli::testing::tempfs();
        fs.write(&rl).unwrap();
        let output = rm
            .run(rl, &base::Config::default(), &fs)
            .unwrap()
            .to_string();
        assert!(
            output.contains(want_in_output),
            "substring `{}` not found in `{}`",
            want_in_output,
            output,
        );
    }
}
