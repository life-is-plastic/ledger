use crate::sharedopts;
use crate::util;

/// View transactions
#[derive(clap::Parser)]
pub struct View {
    #[arg(
        default_value = "m",
        help = sharedopts::INTERVAL_HELP,
        long_help = sharedopts::INTERVAL_LONG_HELP,
    )]
    interval: lib::Interval,

    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl View {
    pub fn run<W>(
        self,
        mut stdout: W,
        rl: lib::Recordlist,
        charset: &lib::Charset,
        config: &lib::Config,
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
        let tr = lib::Tree::from(lib::tree::forview::Config {
            charset,
            first_iid: config.first_index_in_date,
            rl: &rl,
            leaf_string_postprocessor: None,
        });
        if tr.is_empty() {
            util::write_no_transactions_msg(&mut stdout, self.interval)?;
        } else {
            write!(stdout, "{}", tr)?;
        }
        Ok(())
    }
}
