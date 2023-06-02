use anyhow::Context;
use clap::builder::TypedValueParser;

/// View and manage contribution limits
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
    set: Option<lib::Cents>,

    /// View total and remaining limits for YEAR
    #[arg(short, long, value_name = "ACCOUNT_TYPE")]
    #[arg(value_parser(
        clap::builder::PossibleValuesParser::new(<lib::Limitkind as strum::VariantNames>::VARIANTS)
            .map(|s| s.parse::<lib::Limitkind>().expect("should be parseable"))
    ))]
    view: Option<lib::Limitkind>,
}

impl Lim {
    pub fn run<W>(
        self,
        mut stdout: W,
        rl: lib::Recordlist,
        charset: &lib::Charset,
        config: &lib::Config,
        fs: &lib::Fs,
    ) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let year = self.year.0;
        let limits = fs
            .read::<lib::Limits>()
            .with_context(|| format!("failed to read '{}'", fs.path::<lib::Limits>().display()))?;

        if let Some(amount) = self.opts.set {
            return update_limits(&mut stdout, limits, year, amount, fs);
        }

        let kind = get_limitkind(self.opts.view, &config.lim_account_type)?;
        let chart = lib::Limitprinter::from(lib::limitprinter::Config {
            charset,
            today: lib::Date::from_ymd(year, 12, 31).expect("year should be within range"),
            kind,
            limits: &limits,
            rl: &rl,
        });
        write!(stdout, "{}", chart)?;
        Ok(())
    }
}

fn update_limits<W>(
    mut stdout: W,
    mut limits: lib::Limits,
    year: u32,
    amount: lib::Cents,
    fs: &lib::Fs,
) -> anyhow::Result<()>
where
    W: std::io::Write,
{
    let s: String;
    let mut updated = true;
    if amount != lib::Cents(0) {
        limits.set(year, amount);
        s = format!("{} limit set to {}", year, amount);
    } else if limits.remove(year).is_some() {
        limits.remove(year);
        s = format!("{} limit removed.", year);
    } else {
        updated = false;
        s = format!("{} has no limit.", year);
    };
    if updated {
        fs.write(&limits)
            .with_context(|| format!("failed to write '{}'", fs.path::<lib::Limits>().display()))?;
    }
    writeln!(stdout, "{}", s)?;
    Ok(())
}

fn get_limitkind(arg: Option<lib::Limitkind>, default: &str) -> anyhow::Result<lib::Limitkind> {
    if let Some(kind) = arg {
        return Ok(kind);
    }
    if default.is_empty() {
        anyhow::bail!("no default account type configured")
    }
    default
        .parse::<lib::Limitkind>()
        .map_err(|_| anyhow::format_err!("invalid default account type '{}'", default))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct YearArg(u32);

impl std::str::FromStr for YearArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "y" || s == "Y" {
            return Ok(YearArg(lib::Date::today().year()));
        }
        let year = if s.is_empty() || ![b'y', b'Y'].contains(&s.as_bytes()[0]) {
            s.parse()?
        } else {
            let offset = std::str::from_utf8(&s.as_bytes()[1..])
                .expect("remaining bytes should form a valid string")
                .parse::<i32>()?;
            let mut y = (lib::Date::today().year() as i32)
                .checked_add(offset)
                .unwrap_or(10000);
            if y < 0 {
                y = 10000;
            }
            y as u32
        };
        if year > 9999 {
            anyhow::bail!("year is out of range")
        } else {
            Ok(Self(year))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::util::testing::env;
    use crate::util::testing::Env;

    #[rstest]
    #[case("{}", 2015, lib::Cents(0), "{}", "2015 has no limit.\n")]
    #[case(r#"{"2015":1}"#, 2015, lib::Cents(0), "{}", "2015 limit removed.\n")]
    #[case(
        r#"{"2015":1}"#,
        2016,
        lib::Cents(-123),
        r#"{"2015":1,"2016":-123}"#,
        "2016 limit set to (1.23)\n"
    )]
    fn test_update_limits(
        mut env: Env,
        #[case] limits: lib::Limits,
        #[case] year: u32,
        #[case] amount: lib::Cents,
        #[case] want_limits: lib::Limits,
        #[case] want_output: &str,
    ) {
        update_limits(&mut env.stdout, limits, year, amount, &env.fs).unwrap();
        assert_eq!(env.fs.read::<lib::Limits>().unwrap(), want_limits);
        assert_eq!(std::str::from_utf8(&env.stdout).unwrap(), want_output);
    }

    #[rstest]
    #[case(Some(lib::Limitkind::Rrsp), "", lib::Limitkind::Rrsp)]
    #[case(Some(lib::Limitkind::Tfsa), "", lib::Limitkind::Tfsa)]
    #[case(Some(lib::Limitkind::Tfsa), "asdf", lib::Limitkind::Tfsa)]
    #[case(None, "rrsp", lib::Limitkind::Rrsp)]
    #[case(None, "tfsa", lib::Limitkind::Tfsa)]
    fn test_get_limitkind(
        #[case] arg: Option<lib::Limitkind>,
        #[case] default: &str,
        #[case] want: lib::Limitkind,
    ) {
        assert_eq!(get_limitkind(arg, default).unwrap(), want)
    }

    #[rstest]
    #[case(None, "")]
    #[case(None, "asdf")]
    fn test_get_limitkind_failing(#[case] arg: Option<lib::Limitkind>, #[case] default: &str) {
        assert!(get_limitkind(arg, default).is_err())
    }

    #[rstest]
    #[case("0", YearArg(0))]
    #[case("123", YearArg(123))]
    #[case("9999", YearArg(9999))]
    #[case("y", YearArg(lib::Date::today().year()))]
    #[case("Y1", YearArg(lib::Date::today().year() + 1))]
    #[case("y+10", YearArg(lib::Date::today().year() + 10))]
    #[case("Y-10", YearArg(lib::Date::today().year() - 10))]
    fn test_yeararg_from_str(#[case] s: &str, #[case] want: YearArg) {
        assert_eq!(s.parse::<YearArg>().unwrap(), want)
    }

    #[rstest]
    #[case("-1")]
    #[case("10000")]
    #[case("y-9999")]
    #[case("Y+9999")]
    #[case("yy")]
    #[case("a")]
    fn test_yeararg_from_str_failing(#[case] s: &str) {
        assert!(s.parse::<YearArg>().is_err())
    }
}
