use anyhow::Context;

use crate::base;
use crate::cli;

/// Log transactions in a predefined template
#[derive(clap::Parser)]
pub struct Logt {
    /// Transaction template
    ///
    /// If omitted, displays the available templates.
    template: Option<String>,

    /// Transaction date
    #[arg(default_value = "d")]
    date: base::Date,
}

impl Logt {
    pub fn run(
        &self,
        mut rl: base::Recordlist,
        config: &base::Config,
        fs: &base::Fs,
    ) -> anyhow::Result<cli::Output> {
        let Some(tmpl_name) = &self.template else {
            let tr_config = base::tree::forlogt::Config {
                charset: cli::util::charset_from_config(config),
                templates: config.templates.clone(),
            };
            return Ok(cli::Output::TreeForLogt(tr_config));
        };
        let Some(tmpl) = config.templates.get(tmpl_name) else {
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
            charset: cli::util::charset_from_config(config),
            first_iid: config.first_index_in_date,
            leaf_string_postprocessor: None,
            rl,
        };
        Ok(cli::Output::TreeForView(tr_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    cli::testing::generate_testcases![
        (
            nonexistent_template,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "logt", "bad-template"],
                    res: cli::testing::ResultMatcher::ErrGlob("unknown template"),
                }],
                initial_state: cli::testing::StrState::new().with_config("{}"),
            }
        ),
        (
            normal_execution,
            cli::testing::MutCase {
                invocations: &[cli::testing::Invocation {
                    args: &["", "logt", "some_template", "2015-03-30",],
                    res: cli::testing::ResultMatcher::OkExact(cli::Output::TreeForView(
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
                initial_state: cli::testing::StrState::new().with_config(
                    r#"{
                        "templates": {
                            "some_template": [
                                {"category": "gift", "amount": 5000},
                                {"category": "groceries", "amount": -6000}
                            ]
                        }
                    }"#
                ),
                final_state: cli::testing::State::new()
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
