mod cats;
mod init;
mod lim;
mod log;
mod plot;
mod rm;
mod sum;
mod view;

use crate::Output;
use anyhow::Context;

/// Cash flow tracker
#[derive(clap::Parser)]
#[command(color = clap::ColorChoice::Never)]
struct Root {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Init(init::Init),
    Log(log::Log),
    Rm(rm::Rm),
    View(view::View),
    Cats(cats::Cats),
    Sum(sum::Sum),
    Plot(plot::Plot),
    Lim(lim::Lim),
}

impl Root {
    fn run(self, fs: &lib::Fs) -> anyhow::Result<Output> {
        if let Commands::Init(cmd) = self.command {
            return cmd.run(fs);
        }

        if !fs.is_repo() {
            anyhow::bail!("not a repository")
        }
        let config = fs
            .read::<lib::Config>()
            .with_context(|| format!("failed to read '{}'", fs.path::<lib::Config>().display()))?;
        let charset = {
            let mut charset = lib::Charset::default();
            if config.use_unicode_symbols {
                charset = charset.with_unicode()
            }
            if config.use_colored_output {
                charset = charset.with_color()
            }
            charset
        };
        let rl = fs.read::<lib::Recordlist>().with_context(|| {
            format!(
                "failed to read '{}'",
                fs.path::<lib::Recordlist>().display()
            )
        })?;

        match self.command {
            Commands::Init(_) => unreachable!(),
            Commands::Log(cmd) => cmd.run(rl, charset, &config, fs),
            Commands::Rm(cmd) => cmd.run(rl, charset, &config, fs),
            Commands::View(cmd) => cmd.run(rl, charset, &config),
            Commands::Cats(cmd) => cmd.run(rl),
            Commands::Sum(cmd) => cmd.run(rl, charset),
            Commands::Plot(cmd) => cmd.run(rl, charset),
            Commands::Lim(cmd) => cmd.run(rl, charset, &config, fs),
        }
    }
}

pub fn main() {
    fn try_main() -> anyhow::Result<()> {
        let root = <Root as clap::Parser>::parse();
        let cwd = std::env::current_dir().context("failed to resolve current working directory")?;
        let fs = lib::Fs::new(cwd);
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
