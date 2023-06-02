use crate::sharedopts;
use crate::util;

/// View unique categories
#[derive(clap::Parser)]
pub struct Cats {
    #[command(flatten)]
    categories_opts: sharedopts::CategoriesOpts,
}

impl Cats {
    pub fn run<W>(self, mut stdout: W, rl: lib::Recordlist) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let rl = util::filter_rl(
            &rl,
            lib::Interval::MAX,
            &self.categories_opts.categories,
            &self.categories_opts.not_categories,
        );
        let mut cats = rl.iter().map(|r| r.category().str()).collect::<Vec<_>>();
        cats.sort();
        cats.dedup();
        if cats.is_empty() {
            writeln!(stdout, "No transactions.")?;
        } else {
            writeln!(stdout, "{}", cats.join("\n"))?;
        }
        Ok(())
    }
}
