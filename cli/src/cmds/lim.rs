use crate::output::Output;
use crate::util;
use anyhow::Context;
use clap::builder::TypedValueParser;

/// Manage and view contribution limits
#[derive(clap::Parser)]
pub struct Lim {
    /// Year of interest
    ///
    /// May be given in the form 'yn' to indicate an offset of 'n' years from
    /// this year.
    #[arg(default_value = "y", allow_negative_numbers = true)]
    year: YearArg,

    #[command(flatten)]
    opts: Opts,
}

#[derive(clap::Args)]
#[group(required = false, multiple = false)]
struct Opts {
    /// Set the contribution limit for YEAR
    #[arg(short, long, value_name = "AMOUNT", allow_negative_numbers = true)]
    set: Option<base::Cents>,

    /// View total and remaining limits for YEAR
    #[arg(long, value_name = "ACCOUNT_TYPE")]
    #[arg(value_parser(
        clap::builder::PossibleValuesParser::new(<base::Limitkind as strum::VariantNames>::VARIANTS)
            .map(|s| s.parse::<base::Limitkind>().expect("should be parseable"))
    ))]
    view: Option<base::Limitkind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct YearArg(u16);

impl std::str::FromStr for YearArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "y" || s == "Y" {
            return Ok(YearArg(base::Date::today().year()));
        }
        let year = if s.is_empty() || ![b'y', b'Y'].contains(&s.as_bytes()[0]) {
            s.parse()?
        } else {
            let offset = std::str::from_utf8(&s.as_bytes()[1..])
                .expect("remaining bytes should form a valid string")
                .parse::<i32>()?;
            (base::Date::today().year() as i32)
                .checked_add(offset)
                .unwrap_or(-1)
        };
        if !(0..=9999).contains(&year) {
            anyhow::bail!("year is out of range")
        }
        Ok(Self(year as u16))
    }
}

impl Lim {
    pub fn run(
        self,
        rl: base::Recordlist,
        config: &base::Config,
        fs: &base::Fs,
    ) -> anyhow::Result<Output> {
        let year = self.year.0;
        let limits = fs
            .read::<base::Limits>()
            .with_context(|| format!("failed to read '{}'", fs.path::<base::Limits>().display()))?;

        if let Some(amount) = self.opts.set {
            return update_limits(limits, year, amount, fs);
        }

        let Some(kind) = self.opts.view.or(config.lim_account_type) else {
            anyhow::bail!("no default account type configured")
        };
        let printer_config = base::limitprinter::Config {
            charset: util::charset_from_config(config),
            year,
            kind,
            limits,
            rl,
        };
        Ok(Output::Limitprinter(printer_config))
    }
}

fn update_limits(
    mut limits: base::Limits,
    year: u16,
    amount: base::Cents,
    fs: &base::Fs,
) -> anyhow::Result<Output> {
    let output: String;
    let mut updated = true;
    if amount != base::Cents(0) {
        limits.set(year, amount);
        output = format!("{} limit set to {}", year, amount);
    } else if limits.remove(year).is_some() {
        limits.remove(year);
        output = format!("{} limit removed.", year);
    } else {
        updated = false;
        output = format!("{} has no limit.", year);
    };
    if updated {
        fs.write(&limits).with_context(|| {
            format!("failed to write '{}'", fs.path::<base::Limits>().display())
        })?;
    }
    Ok(Output::Str(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use rstest::rstest;

    #[rstest]
    #[case("0", YearArg(0))]
    #[case("123", YearArg(123))]
    #[case("9999", YearArg(9999))]
    #[case("y", YearArg(base::Date::today().year()))]
    #[case("Y1", YearArg(base::Date::today().year() + 1))]
    #[case("y+10", YearArg(base::Date::today().year() + 10))]
    #[case("Y-10", YearArg(base::Date::today().year() - 10))]
    fn test_yeararg_from_str(#[case] s: &str, #[case] want: YearArg) {
        assert_eq!(s.parse::<YearArg>().unwrap(), want)
    }

    #[rstest]
    #[case("-1")]
    #[case("10000")]
    #[case("y-9999")]
    #[case("Y+9999")]
    #[case("y-99999999999999999999999999999999999999999999999")]
    #[case("yy")]
    #[case("a")]
    fn test_yeararg_from_str_failing(#[case] s: &str) {
        assert!(s.parse::<YearArg>().is_err())
    }

    testing::generate_testcases![
        (
            remove_nonexistent,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "lim", "2015", "--set", "0"],
                    res: testing::ResultMatcher::OkStrGlob("2015 has no limit."),
                }],
                initial_state: testing::StrState::new().with_config("{}"),
            }
        ),
        (
            remove,
            testing::MutCase {
                invocations: &[testing::Invocation {
                    args: &["", "lim", "2015", "--set", "0"],
                    res: testing::ResultMatcher::OkStrGlob("2015 limit removed."),
                }],
                initial_state: testing::StrState::new()
                    .with_config("{}")
                    .with_limits(r#"{"2015":1}"#),
                final_state: testing::State::new()
                    .with_config(base::Config::default())
                    .with_limits(base::Limits::new()),
            }
        ),
        (
            set,
            testing::MutCase {
                invocations: &[testing::Invocation {
                    args: &["", "lim", "2016", "--set=-1.23"],
                    res: testing::ResultMatcher::OkStrGlob("2016 limit set to (1.23)"),
                }],
                initial_state: testing::StrState::new()
                    .with_config("{}")
                    .with_limits(r#"{"2015":1}"#),
                final_state: testing::State::new()
                    .with_config(base::Config::default())
                    .with_limits(r#"{"2015":1,"2016":-123}"#),
            }
        ),
        (
            view_explicit_limitkind,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "lim", "2015", "--view", "tfsa"],
                    res: testing::ResultMatcher::OkExact(Output::Limitprinter(
                        base::limitprinter::Config {
                            charset: Default::default(),
                            year: 2015,
                            kind: base::Limitkind::Tfsa,
                            limits: r#"{
                                "2014": 100,
                                "2015": 100
                            }"#
                            .parse()
                            .unwrap(),
                            rl: r#"
                                {"d":"2014-01-01","c":"aaa","a":100}
                                {"d":"2015-01-01","c":"aaa","a":100}
                            "#
                            .parse()
                            .unwrap(),
                        }
                    )),
                }],
                initial_state: testing::StrState::default()
                    .with_config("{}")
                    .with_limits(
                        r#"{
                            "2014": 100,
                            "2015": 100
                        }"#
                    )
                    .with_rl(
                        r#"
                            {"d":"2014-01-01","c":"aaa","a":100}
                            {"d":"2015-01-01","c":"aaa","a":100}
                        "#
                    ),
            }
        ),
        (
            view_implicit_limitkind,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "lim"],
                    res: testing::ResultMatcher::OkExact(Output::Limitprinter(
                        base::limitprinter::Config {
                            charset: Default::default(),
                            year: base::Date::today().year(),
                            kind: base::Limitkind::Tfsa,
                            limits: r#"{
                                "2014": 100,
                                "2015": 100
                            }"#
                            .parse()
                            .unwrap(),
                            rl: r#"
                                {"d":"2014-01-01","c":"aaa","a":100}
                                {"d":"2015-01-01","c":"aaa","a":100}
                            "#
                            .parse()
                            .unwrap(),
                        }
                    )),
                }],
                initial_state: testing::StrState::new()
                    .with_config(r#"{"limAccountType":"tfsa"}"#)
                    .with_limits(
                        r#"{
                            "2014": 100,
                            "2015": 100
                        }"#
                    )
                    .with_rl(
                        r#"
                            {"d":"2014-01-01","c":"aaa","a":100}
                            {"d":"2015-01-01","c":"aaa","a":100}
                        "#
                    ),
            }
        ),
        (
            view_implicit_limitkind_without_one_being_configured,
            testing::Case {
                invocations: &[testing::Invocation {
                    args: &["", "lim"],
                    res: testing::ResultMatcher::ErrGlob("no default account type configured"),
                }],
                initial_state: testing::StrState::new().with_config("{}"),
            }
        ),
    ];
}
