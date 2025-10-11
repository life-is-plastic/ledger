use crate::base;
use crate::cli;

/// View transaction totals
#[derive(clap::Parser)]
pub struct Sum {
    #[arg(
        default_value = "m",
        help = cli::sharedopts::INTERVAL_HELP,
        long_help = cli::sharedopts::INTERVAL_HELP_LONG,
    )]
    interval: base::Interval,

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
    categories_opts: cli::sharedopts::CategoriesOpts,
}

impl Sum {
    pub fn run(&self, rl: base::Recordlist, config: &base::Config) -> anyhow::Result<cli::Output> {
        let categories = cli::util::preprocess_categories(
            &self.categories_opts.categories,
            self.categories_opts.fullmatch,
        );
        let not_categories = cli::util::preprocess_categories(
            &self.categories_opts.not_categories,
            self.categories_opts.fullmatch,
        );
        let rl = cli::util::filter_rl(&rl, self.interval, &categories, &not_categories);
        let tr_config = base::tree::forsum::Config {
            charset: cli::util::charset_from_config(config),
            level: self.level,
            rl,
        };
        Ok(cli::Output::TreeForSum(tr_config, self.interval))
    }
}
