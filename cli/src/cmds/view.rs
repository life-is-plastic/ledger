use crate::output::Output;
use crate::sharedopts;
use crate::util;

/// View transactions
#[derive(clap::Parser)]
pub struct View {
    #[arg(
        default_value = "m",
        help = sharedopts::INTERVAL_HELP,
        long_help = sharedopts::INTERVAL_HELP_LONG,
    )]
    interval: base::Interval,

    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl View {
    pub fn run(self, rl: base::Recordlist, config: &base::Config) -> anyhow::Result<Output> {
        let rl = util::filter_rl(
            &rl,
            self.interval,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let tr_config = base::tree::forview::Config {
            charset: util::charset_from_config(config),
            first_iid: config.first_index_in_date,
            rl,
            leaf_string_postprocessor: None,
        };
        Ok(Output::TreeForView(tr_config))
    }
}
