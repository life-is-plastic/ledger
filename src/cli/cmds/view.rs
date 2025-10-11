use crate::base;
use crate::cli;

/// View transactions
#[derive(clap::Parser)]
pub struct View {
    #[arg(
        default_value = "m",
        help = cli::sharedopts::INTERVAL_HELP,
        long_help = cli::sharedopts::INTERVAL_HELP_LONG,
    )]
    interval: base::Interval,

    #[command(flatten)]
    categories_opts: cli::sharedopts::CategoriesOpts,
}

impl View {
    pub fn run(self, rl: base::Recordlist, config: &base::Config) -> anyhow::Result<cli::Output> {
        let categories = cli::util::preprocess_categories(
            &self.categories_opts.categories,
            self.categories_opts.fullmatch,
        );
        let not_categories = cli::util::preprocess_categories(
            &self.categories_opts.not_categories,
            self.categories_opts.fullmatch,
        );
        let rl = cli::util::filter_rl(&rl, self.interval, &categories, &not_categories);
        let tr_config = base::tree::forview::Config {
            charset: cli::util::charset_from_config(config),
            first_iid: config.first_index_in_date,
            rl,
            leaf_string_postprocessor: None,
        };
        Ok(cli::Output::TreeForView(tr_config))
    }
}
