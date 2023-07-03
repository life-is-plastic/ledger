use crate::sharedopts;
use crate::util;
use crate::Output;

/// View transactions
#[derive(clap::Parser)]
pub struct View {
    #[arg(
        default_value = "m",
        help = sharedopts::INTERVAL_HELP,
        long_help = sharedopts::INTERVAL_HELP_LONG,
    )]
    interval: lib::Interval,

    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl View {
    pub fn run(
        self,
        rl: lib::Recordlist,
        charset: lib::Charset,
        config: &lib::Config,
    ) -> anyhow::Result<Output> {
        let rl = util::filter_rl(
            &rl,
            self.interval,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let tr_config = lib::tree::forview::Config {
            charset,
            first_iid: config.first_index_in_date,
            rl,
            leaf_string_postprocessor: None,
        };
        Ok(Output::TreeForView(tr_config))
    }
}
