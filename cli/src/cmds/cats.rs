use crate::util;
use crate::Output;

/// View unique categories
#[derive(clap::Parser)]
pub struct Cats {
    /// Wildcard pattern to match categories of interest
    ///
    /// If multiple patterns are provided, include categories that match any
    /// pattern.
    #[arg(default_value = "*")]
    pub category: Vec<String>,
}

impl Cats {
    pub fn run(self, rl: lib::Recordlist) -> anyhow::Result<Output> {
        let rl = util::filter_rl::<_, &str>(&rl, lib::Interval::MAX, &self.category, &[]);
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
