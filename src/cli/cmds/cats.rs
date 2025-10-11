use crate::base;
use crate::cli;

/// View unique categories
#[derive(clap::Parser)]
pub struct Cats {
    #[arg(long, help = cli::sharedopts::FULLMATCH_HELP, long_help = cli::sharedopts::FULLMATCH_HELP_LONG)]
    pub fullmatch: bool,

    /// Wildcard pattern to match categories of interest
    ///
    /// If multiple patterns are provided, include categories that match any
    /// pattern.
    #[arg(default_value = "*")]
    pub category: Vec<String>,
}

impl Cats {
    pub fn run(&self, rl: base::Recordlist) -> anyhow::Result<cli::Output> {
        let categories = cli::util::preprocess_categories(&self.category, self.fullmatch);
        let rl = cli::util::filter_rl::<_, &str>(&rl, base::Interval::MAX, &categories, &[]);
        let mut cats = rl.iter().map(|r| r.category().as_str()).collect::<Vec<_>>();
        cats.sort();
        cats.dedup();
        Ok(if cats.is_empty() {
            cli::Output::Str("No categories.".to_string())
        } else {
            cli::Output::Str(cats.join("\n"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    cli::testing::generate_testcases![
        (
            no_cats1,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "cats"],
                    res: cli::testing::ResultMatcher::OkStrGlob("no categories."),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}"),
            }
        ),
        (
            no_cats2,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "cats", "ddd"],
                    res: cli::testing::ResultMatcher::OkStrGlob("no categories."),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"2014-01-01","c":"ccc","a":100}
                        {"d":"2015-01-01","c":"bbb","a":100}
                        {"d":"2016-01-01","c":"aaa","a":100}
                    "#
                ),
            }
        ),
        (
            normal_execution,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "cats", "bbb", "aaa"],
                    res: cli::testing::ResultMatcher::OkStrGlob("aaa\nbbb"),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"2014-01-01","c":"ccc","a":100}
                        {"d":"2015-01-01","c":"bbb","a":100}
                        {"d":"2016-01-01","c":"aaa","a":100}
                    "#
                ),
            }
        ),
        (
            fullmatch_off,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "cats", "*b", "aa"],
                    res: cli::testing::ResultMatcher::OkStrGlob("aaa\nbbb"),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"2014-01-01","c":"ccc","a":100}
                        {"d":"2015-01-01","c":"bbb","a":100}
                        {"d":"2016-01-01","c":"aaa","a":100}
                    "#
                ),
            }
        ),
        (
            fullmatch_on,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "cats", "--fullmatch", "*b", "aa"],
                    res: cli::testing::ResultMatcher::OkStrGlob("bbb"),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}").with_rl(
                    r#"
                        {"d":"2014-01-01","c":"ccc","a":100}
                        {"d":"2015-01-01","c":"bbb","a":100}
                        {"d":"2016-01-01","c":"aaa","a":100}
                    "#
                ),
            }
        ),
    ];
}
