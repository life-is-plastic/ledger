use clap::builder::TypedValueParser;

use crate::sharedopts;
use crate::util;

/// Plot transaction totals
#[derive(clap::Parser)]
pub struct Plot {
    #[arg(
        default_value = "m-12:M",
        help = sharedopts::INTERVAL_HELP,
        long_help = sharedopts::INTERVAL_LONG_HELP,
    )]
    interval: lib::Interval,

    #[arg(
        short,
        long,
        default_value_t = lib::Datepart::Month,
        value_parser(
            clap::builder::PossibleValuesParser::new(<lib::Datepart as strum::VariantNames>::VARIANTS)
                .map(|s| s.parse::<lib::Datepart>().expect("should be parseable"))
        ),
    )]
    unit: lib::Datepart,

    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl Plot {
    pub fn run<W>(
        self,
        mut stdout: W,
        rl: lib::Recordlist,
        charset: &lib::Charset,
    ) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let rl = util::filter_rl(
            &rl,
            self.interval,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let chart = lib::Barchart::from(lib::barchart::Config {
            charset,
            bounds: self.interval,
            unit: self.unit,
            term_width: terminal_size::terminal_size()
                .map(|(w, _)| w.0)
                .unwrap_or_default() as usize,
            rl: &rl,
        });
        if chart.is_empty() {
            util::write_no_transactions_msg(&mut stdout, self.interval)?;
        } else {
            write!(stdout, "{}", chart)?;
        }
        Ok(())
    }
}
