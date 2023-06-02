mod cats;
mod init;
mod lim;
mod log;
mod plot;
mod rm;
mod sum;
mod view;

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
    fn run<W>(self, mut stdout: W, fs: &lib::Fs) -> anyhow::Result<()>
    where
        W: std::io::Write,
    {
        let root = <Self as clap::Parser>::parse();
        if let Commands::Init(cmd) = root.command {
            return cmd.run(&mut stdout, fs);
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

        match root.command {
            Commands::Init(_) => unreachable!(),
            Commands::Log(cmd) => cmd.run(&mut stdout, rl, &charset, &config, fs),
            Commands::Rm(cmd) => cmd.run(&mut stdout, rl, &charset, &config, fs),
            Commands::View(cmd) => cmd.run(&mut stdout, rl, &charset, &config),
            Commands::Cats(cmd) => cmd.run(&mut stdout, rl),
            Commands::Sum(cmd) => cmd.run(&mut stdout, rl, &charset),
            Commands::Plot(cmd) => cmd.run(&mut stdout, rl, &charset),
            Commands::Lim(cmd) => cmd.run(&mut stdout, rl, &charset, &config, fs),
        }
    }
}

pub fn main() {
    fn try_main() -> anyhow::Result<()> {
        let root = <Root as clap::Parser>::parse();
        let mut stdout = std::io::stdout();
        let cwd = std::env::current_dir().context("failed to resolve current working directory")?;
        let fs = lib::Fs::new(cwd);

        let res = root.run(&stdout, &fs);
        <std::io::Stdout as std::io::Write>::flush(&mut stdout)?;
        res
    }

    if let Err(e) = try_main() {
        eprint!("error");
        e.chain().for_each(|cause| eprint!(": {}", cause));
        eprintln!();
        std::process::exit(1);
    }
}
