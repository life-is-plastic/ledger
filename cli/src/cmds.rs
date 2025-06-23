mod cats;
mod init;
mod lim;
mod log;
mod logt;
mod plot;
mod rm;
mod sum;
mod view;

use crate::output::Output;
use anyhow::Context;

/// Cash flow tracker
#[derive(clap::Parser)]
#[command(color = clap::ColorChoice::Never)]
pub struct Root {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init(init::Init),
    Log(log::Log),
    Logt(logt::Logt),
    Rm(rm::Rm),
    View(view::View),
    Cats(cats::Cats),
    Sum(sum::Sum),
    Plot(plot::Plot),
    Lim(lim::Lim),
}

impl Root {
    pub fn run(self, fs: &base::Fs) -> anyhow::Result<Output> {
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

pub fn main() {
    fn try_main() -> anyhow::Result<()> {
        let root = <Root as clap::Parser>::parse();
        let cwd = std::env::current_dir().context("failed to resolve current working directory")?;
        let fs = base::Fs::new(cwd);
        let output = root.run(&fs)?;
        print!("{}", output);
        Ok(())
    }

    if let Err(e) = try_main() {
        eprint!("error");
        e.chain().for_each(|cause| eprint!(": {}", cause));
        eprintln!();
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;
    use rstest::rstest;

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
