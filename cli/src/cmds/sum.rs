use crate::sharedopts;
use crate::util;
use crate::Output;

/// View transaction totals
#[derive(clap::Parser)]
pub struct Sum {
    #[arg(
        default_value = "m",
        help = sharedopts::INTERVAL_HELP,
        long_help = sharedopts::INTERVAL_HELP_LONG,
    )]
    interval: lib::Interval,

    /// Category level to aggregate on
    ///
    /// Examples:
    /// LEVEL = 2: commute/car/gas -> commute/car
    /// LEVEL = 2: commute/car -> commute/car
    /// LEVEL = 2: commute -> commute
    /// LEVEL = 0: commute -> All
    /// LEVEL = 0: some/other/category -> All
    #[arg(short, long, default_value_t = 1, verbatim_doc_comment)]
    level: usize,

    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl Sum {
    pub fn run(self, rl: lib::Recordlist, config: &lib::Config) -> anyhow::Result<Output> {
        let rl = util::filter_rl(
            &rl,
            self.interval,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let tr_config = lib::tree::forsum::Config {
            charset: util::charset_from_config(config),
            level: self.level,
            rl,
        };
        Ok(Output::TreeForSum(tr_config, self.interval))
    }
}
