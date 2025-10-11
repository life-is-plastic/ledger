mod base;
mod cli;

use anyhow::Context;

fn main() {
    fn try_main() -> anyhow::Result<()> {
        let root = <cli::Root as clap::Parser>::parse();
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
