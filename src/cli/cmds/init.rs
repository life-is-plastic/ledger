use anyhow::Context;

use crate::base;
use crate::cli;

/// Initialize reposistory in the current directory
#[derive(clap::Parser)]
pub struct Init {
    /// Restore an existing repository's config to defaults
    #[arg(long)]
    reset_config: bool,
}

fn initial_config() -> base::Config {
    base::Config {
        first_index_in_date: 1,
        use_colored_output: true,
        use_unicode_symbols: true,
        ..Default::default()
    }
}

impl Init {
    pub fn run(&self, fs: &base::Fs) -> anyhow::Result<cli::Output> {
        let already_repo = fs.is_repo();

        let path = fs.path::<base::Config>();
        let config = if self.reset_config || !path.exists() {
            initial_config()
        } else {
            fs.read::<base::Config>()
                .with_context(|| format!("failed to read '{}'", path.display()))?
        };
        fs.write(&config)
            .with_context(|| format!("failed to write '{}'", path.display()))?;

        Ok(if !already_repo {
            cli::Output::Str(format!(
                "Repository initialized in '{}'",
                fs.dir().display()
            ))
        } else if self.reset_config {
            cli::Output::Str("Repository configuration reset to defaults.".to_string())
        } else {
            cli::Output::Str(format!(
                "Repository reinitialized in '{}'",
                fs.dir().display()
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    cli::testing::generate_testcases![
        (
            empty_repo,
            cli::testing::MutCase {
                invocations: &[cli::testing::Invocation {
                    args: &["", "init"],
                    res: cli::testing::ResultMatcher::OkStrGlob("repository initialized in*"),
                }],
                initial_state: cli::testing::StrState::new(),
                final_state: cli::testing::State::new().with_config(initial_config()),
            }
        ),
        (
            empty_repo_reset_config,
            cli::testing::MutCase {
                invocations: &[cli::testing::Invocation {
                    args: &["", "init", "--reset-config"],
                    res: cli::testing::ResultMatcher::OkStrGlob("repository initialized in*"),
                }],
                initial_state: cli::testing::StrState::new(),
                final_state: cli::testing::State::new().with_config(initial_config()),
            }
        ),
        (
            existing_repo,
            cli::testing::Case {
                invocations: &[cli::testing::Invocation {
                    args: &["", "init"],
                    res: cli::testing::ResultMatcher::OkStrGlob("repository reinitialized in*"),
                }],
                initial_state: cli::testing::StrState::new()
                    .with_config(r#"{"firstIndexInDate":4,"useColoredOutput":true}"#),
            }
        ),
        (
            existing_repo_reset_config,
            cli::testing::MutCase {
                invocations: &[cli::testing::Invocation {
                    args: &["", "init", "--reset-config"],
                    res: cli::testing::ResultMatcher::OkStrGlob(
                        "repository configuration reset to defaults."
                    ),
                }],
                initial_state: cli::testing::StrState::new()
                    .with_config(r#"{"firstIndexInDate":4,"useColoredOutput":true}"#),
                final_state: cli::testing::State::new().with_config(initial_config()),
            }
        ),
    ];
}
