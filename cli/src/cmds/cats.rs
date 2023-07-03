use crate::sharedopts;
use crate::util;
use crate::Output;

/// View unique categories
#[derive(clap::Parser)]
pub struct Cats {
    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl Cats {
    pub fn run(self, rl: lib::Recordlist) -> anyhow::Result<Output> {
        let rl = util::filter_rl(
            &rl,
            lib::Interval::MAX,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let mut cats = rl.iter().map(|r| r.category().as_str()).collect::<Vec<_>>();
        cats.sort();
        cats.dedup();
        Ok(if cats.is_empty() {
            Output::Str("No categories.")
        } else {
            Output::String(cats.join("\n"))
        })
    }
}
