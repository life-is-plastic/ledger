use crate::Output;
use anyhow::Context;

/// Initialize reposistory in the current directory
#[derive(clap::Parser)]
pub struct Init {
    /// Restore an existing repository's config to defaults
    #[arg(long)]
    reset_config: bool,
}

impl Init {
    pub fn run(self, fs: &lib::Fs) -> anyhow::Result<Output> {
        let already_repo = fs.is_repo();

        let config = if self.reset_config {
            lib::Config::default()
        } else {
            fs.read::<lib::Config>().with_context(|| {
                format!("failed to read '{}'", fs.path::<lib::Config>().display())
            })?
        };
        fs.write(&config)
            .with_context(|| format!("failed to write '{}'", fs.path::<lib::Config>().display()))?;

        Ok(if !already_repo {
            Output::String(format!(
                "Repository initialized in '{}'",
                fs.dir().display()
            ))
        } else if self.reset_config {
            Output::Str("Repository configuration reset to defaults.")
        } else {
            Output::String(format!(
                "Repository reinitialized in '{}'",
                fs.dir().display()
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::util::testing::env;
    use crate::util::testing::Env;

    #[rstest]
    #[case(Init { reset_config: false })]
    #[case(Init { reset_config: true })]
    fn test_new_repo(env: Env, #[case] init: Init) {
        let output = init.run(&env.fs).unwrap().to_string();
        assert!(output.starts_with("Repository initialized in"));
        assert_eq!(
            std::fs::read_to_string(env.fs.path::<lib::Config>()).unwrap(),
            lib::Config::default().to_string()
        );
    }

    #[rstest]
    #[case(
        Init { reset_config: false },
        r#"{"firstIndexInDate":4,"useColoredOutput":false}"#,
        lib::Config {
            first_index_in_date: 4,
            use_colored_output: false,
            ..lib::Config::default()
        }.to_string(),
        "Repository reinitialized in",
    )]
    #[case(
        Init { reset_config: true },
        r#"{"firstIndexInDate":4,"useColoredOutput":false}"#,
        lib::Config::default().to_string(),
        "Repository configuration reset to defaults.",
    )]
    fn test_existing_repo(
        env: Env,
        #[case] init: Init,
        #[case] initial_config_contents: &str,
        #[case] final_config_contents: String,
        #[case] want_output_starts_with: &str,
    ) {
        std::fs::write(env.fs.path::<lib::Config>(), initial_config_contents).unwrap();
        let output = init.run(&env.fs).unwrap().to_string();
        assert!(output.starts_with(want_output_starts_with));
        assert_eq!(
            std::fs::read_to_string(env.fs.path::<lib::Config>()).unwrap(),
            final_config_contents,
        )
    }
}
