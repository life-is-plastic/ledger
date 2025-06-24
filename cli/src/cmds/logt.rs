use anyhow::Context;

use crate::output::Output;
use crate::util;

/// Log transactions in a predefined template
#[derive(clap::Parser)]
pub struct Logt {
    /// Transaction template
    template: String,

    /// Transaction date
    #[arg(default_value = "d")]
    date: base::Date,
}

impl Logt {
    pub fn run(
        self,
        mut rl: base::Recordlist,
        config: &base::Config,
        fs: &base::Fs,
    ) -> anyhow::Result<Output> {
        let Some(tmpl) = config.templates.get(&self.template) else {
            anyhow::bail!("unknown template");
        };
        for entry in tmpl {
            let r = base::Record::new(
                self.date,
                entry.category.clone(),
                entry.amount,
                String::new(),
            );
            rl.insert(r);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    testing::generate_testcases![
        (
            nonexistent_template,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "logt", "bad-template"],
                    res: testing::ResultMatcher::ErrGlob("unknown template"),
                }],
                initial_state: testing::StrState::new().with_config("{}"),
            }
        ),
        (
            normal_execution,
            testing::MutCase {
                invocations: &[testing::Invocation {
                    args: &["", "logt", "some_template", "2015-03-30",],
                    res: testing::ResultMatcher::OkExact(Output::TreeForView(
                        base::tree::forview::Config {
                            charset: Default::default(),
                            first_iid: 0,
                            rl: r#"
                                {"d":"2015-03-30","c":"gift","a":5000,"n":""}
                                {"d":"2015-03-30","c":"groceries","a":-6000,"n":""}
                            "#
                            .parse()
                            .unwrap(),
                            leaf_string_postprocessor: None,
                        }
                    )),
                },],
                initial_state: testing::StrState::new().with_config(
                    r#"{
                        "templates": {
                            "some_template": [
                                {"category": "gift", "amount": 5000},
                                {"category": "groceries", "amount": -6000}
                            ]
                        }
                    }"#
                ),
                final_state: testing::State::new()
                    .with_config(
                        r#"{
                            "templates": {
                                "some_template": [
                                    {"category": "gift", "amount": 5000},
                                    {"category": "groceries", "amount": -6000}
                                ]
                            }
                        }"#
                    )
                    .with_rl(
                        r#"
                            {"d":"2015-03-30","c":"gift","a":5000,"n":""}
                            {"d":"2015-03-30","c":"groceries","a":-6000,"n":""}
                        "#
                    ),
            }
        ),
    ];
}
