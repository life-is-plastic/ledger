use crate::sharedopts;
use crate::util;
use crate::Output;

/// Plot transaction totals
#[derive(clap::Parser)]
pub struct Plot {
    #[arg(help = sharedopts::INTERVAL_HELP, long_help = sharedopts::INTERVAL_HELP_LONG)]
    interval: Option<lib::Interval>,

    #[command(flatten)]
    units: Units,

    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

#[derive(clap::Args)]
#[group(required = false, multiple = false)]
struct Units {
    /// Aggregate data by day
    ///
    /// The default interval is the past 2 weeks
    #[arg(short)]
    d: bool,

    /// Aggregate data by month [default]
    ///
    /// The default interval is the past 12 months
    #[arg(short)]
    m: bool,

    /// Aggregate data by year
    ///
    /// The default interval is the past 10 years
    #[arg(short)]
    y: bool,
}

impl Plot {
    pub fn run(self, rl: lib::Recordlist, config: &lib::Config) -> anyhow::Result<Output> {
        let unit = if self.units.y {
            lib::Datepart::Year
        } else if self.units.m {
            lib::Datepart::Month
        } else if self.units.d {
            lib::Datepart::Day
        } else {
            lib::Datepart::Month
        };
        let interval = self.interval.unwrap_or_else(|| {
            let default = match unit {
                lib::Datepart::Year => "y-10:Y",
                lib::Datepart::Month => "m-12:M",
                lib::Datepart::Day => "d-14:D",
            };
            default
                .parse()
                .expect("value should be convertible to Interval object")
        });
        let rl = util::filter_rl(
            &rl,
            interval,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let chart_config = lib::barchart::Config {
            charset: util::charset_from_config(config),
            bounds: interval,
            unit,
            term_width: terminal_size::terminal_size()
                .map(|(w, _)| w.0)
                .unwrap_or_default() as usize,
            rl,
        };
        Ok(Output::Barchart(chart_config))
    }
}
