use anyhow::Context;

use crate::base;
use crate::cli;

/// Cash flow tracker
#[derive(clap::Parser)]
#[command(color = clap::ColorChoice::Never)]
pub struct Root {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init(cli::cmds::init::Init),
    Log(cli::cmds::log::Log),
    Logt(cli::cmds::logt::Logt),
    Rm(cli::cmds::rm::Rm),
    View(cli::cmds::view::View),
    Cats(cli::cmds::cats::Cats),
    Sum(cli::cmds::sum::Sum),
    Plot(cli::cmds::plot::Plot),
    Lim(cli::cmds::lim::Lim),
}

impl Root {
    pub fn run(self, fs: &base::Fs) -> anyhow::Result<cli::Output> {
        if let Commands::Init(cmd) = self.command {
            return cmd.run(fs);
        }

        if !fs.is_repo() {
            anyhow::bail!("not a repository")
        }
        let config = fs
            .read::<base::Config>()
            .with_context(|| format!("failed to read '{}'", fs.path::<base::Config>().display()))?;
        let rl = fs.read::<base::Recordlist>().with_context(|| {
            format!(
                "failed to read '{}'",
                fs.path::<base::Recordlist>().display()
            )
        })?;

        match self.command {
            Commands::Init(_) => unreachable!(),
            Commands::Log(cmd) => cmd.run(rl, &config, fs),
            Commands::Logt(cmd) => cmd.run(rl, &config, fs),
            Commands::Rm(cmd) => cmd.run(rl, &config, fs),
            Commands::View(cmd) => cmd.run(rl, &config),
            Commands::Cats(cmd) => cmd.run(rl),
            Commands::Sum(cmd) => cmd.run(rl, &config),
            Commands::Plot(cmd) => cmd.run(rl, &config),
            Commands::Lim(cmd) => cmd.run(rl, &config, fs),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::cli::testing;

    #[rstest]
    #[case(&["", "log", "aaa", "123"])]
    #[case(&["", "rm", "d", "0"])]
    #[case(&["", "view"])]
    #[case(&["", "cats"])]
    #[case(&["", "sum"])]
    #[case(&["", "plot"])]
    #[case(&["", "lim", "--set", "0"])]
    fn test_error_if_not_a_repo(#[case] args: &[&str]) {
        let (fs, _td) = testing::tempfs();
        let root = match <Root as clap::Parser>::try_parse_from(args) {
            Ok(cmd) => cmd,
            Err(e) => panic!("{}", e),
        };
        let res = root.run(&fs);
        assert!(matches!(res, Err(ref e) if e.to_string() == "not a repository"))
    }
}
