use crate::util;
use crate::Output;
use anyhow::Context;

/// Log a transaction
#[derive(clap::Parser)]
pub struct Log {
    /// Transaction category, case-sensitive
    ///
    /// Use '/' to indicate hierarchy. For example, in 'commute/car/gas',
    /// 'commute' is the top level category, 'commute/car' is the second level,
    /// and 'commute/car/gas' is the leaf.
    category: lib::Category,

    /// Transaction amount
    ///
    /// Note that although the value 0 is allowed, such transactions are
    /// effectively ignored by commands 'sum', 'plot', and 'lim'.
    #[arg(allow_negative_numbers = true)]
    amount: CentsArg,

    /// Transaction date
    #[arg(default_value = "d")]
    date: lib::Date,

    /// Optional comments about transaction
    #[arg(short, long, default_value_t, hide_default_value = true)]
    note: String,
}

impl Log {
    pub fn run(
        self,
        mut rl: lib::Recordlist,
        config: &lib::Config,
        fs: &lib::Fs,
    ) -> anyhow::Result<Output> {
        let r = lib::Record::new(
            self.date,
            self.category,
            self.amount.into_cents(config.unsigned_is_positive),
            self.note,
        );
        rl.insert(r);
        fs.write(&rl).with_context(|| {
            format!(
                "failed to write '{}'",
                fs.path::<lib::Recordlist>().display()
            )
        })?;
        let rl = rl
            .slice_spanning_interval(lib::Interval {
                start: self.date,
                end: self.date,
            })
            .iter()
            .collect::<lib::Recordlist>();
        let tr_config = lib::tree::forview::Config {
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
    Signed(lib::Cents),
    Unsigned(lib::Cents),
}

impl CentsArg {
    fn into_cents(self, unsigned_is_positive: bool) -> lib::Cents {
        match self {
            CentsArg::Signed(x) => x,
            CentsArg::Unsigned(x) => {
                if unsigned_is_positive {
                    x.0.abs().into()
                } else {
                    (-x.0.abs()).into()
                }
            }
        }
    }
}

impl std::str::FromStr for CentsArg {
    type Err = <lib::Cents as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cents = lib::Cents::from_str(s)?;
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
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(CentsArg::Signed(lib::Cents(123)), true, lib::Cents(123))]
    #[case(CentsArg::Signed(lib::Cents(123)), false, lib::Cents(123))]
    #[case(CentsArg::Signed(lib::Cents(-123)), true, lib::Cents(-123))]
    #[case(CentsArg::Signed(lib::Cents(-123)), false, lib::Cents(-123))]
    #[case(CentsArg::Unsigned(lib::Cents(123)), true, lib::Cents(123))]
    #[case(CentsArg::Unsigned(lib::Cents(123)), false, lib::Cents(-123))]
    #[case(CentsArg::Unsigned(lib::Cents(-123)), true, lib::Cents(123))]
    #[case(CentsArg::Unsigned(lib::Cents(-123)), false, lib::Cents(-123))]
    fn test_centsarg_into_cents(
        #[case] arg: CentsArg,
        #[case] unsigned_is_positive: bool,
        #[case] want: lib::Cents,
    ) {
        assert_eq!(arg.into_cents(unsigned_is_positive), want)
    }
}
