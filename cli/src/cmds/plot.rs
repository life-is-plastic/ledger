use crate::sharedopts;
use crate::util;
use crate::Output;
use clap::builder::TypedValueParser;

/// Plot transaction totals
#[derive(clap::Parser)]
pub struct Plot {
    #[arg(
        default_value = "m-12:M",
        help = sharedopts::INTERVAL_HELP,
        long_help = sharedopts::INTERVAL_HELP_LONG,
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
    pub fn run(self, rl: lib::Recordlist, config: &lib::Config) -> anyhow::Result<Output> {
        let rl = util::filter_rl(
            &rl,
            self.interval,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let chart_config = lib::barchart::Config {
            charset: util::charset_from_config(config),
            bounds: self.interval,
            unit: self.unit,
            term_width: term_width(),
            rl,
        };
        Ok(Output::Barchart(chart_config))
    }
}

fn term_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0)
        .unwrap_or_default() as usize
}
