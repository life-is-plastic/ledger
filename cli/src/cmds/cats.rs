use crate::output::Output;
use crate::util;

/// View unique categories
#[derive(clap::Parser)]
pub struct Cats {
    /// Wildcard pattern to match categories of interest
    ///
    /// If multiple patterns are provided, include categories that match any
    /// pattern.
    #[arg(default_value = "*")]
    pub category: Vec<String>,
}

impl Cats {
    pub fn run(self, rl: base::Recordlist) -> anyhow::Result<Output> {
        let rl = util::filter_rl::<_, &str>(&rl, base::Interval::MAX, &self.category, &[]);
        let mut cats = rl.iter().map(|r| r.category().as_str()).collect::<Vec<_>>();
        cats.sort();
        cats.dedup();
        Ok(if cats.is_empty() {
            Output::Str("No categories.".to_string())
        } else {
            Output::Str(cats.join("\n"))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;

    testing::generate_testcases![
        (
            no_cats1,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "cats"],
                    res: testing::ResultMatcher::OkStrGlob("no categories."),
                }],
                initial_state: testing::StrState::new().with_config("{}"),
            }
        ),
        (
            no_cats2,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "cats", "ddd"],
                    res: testing::ResultMatcher::OkStrGlob("no categories."),
                }],
                initial_state: testing::StrState::new().with_config("{}").with_rl(
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
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "cats", "bbb", "aaa"],
                    res: testing::ResultMatcher::OkStrGlob("aaa\nbbb"),
                }],
                initial_state: testing::StrState::new().with_config("{}").with_rl(
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
