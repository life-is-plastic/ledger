/// Output of a successful command invocation, to be written to stdout.
pub enum Output {
    Str(&'static str),
    String(String),
    TreeForSum(lib::tree::forsum::Config),
    TreeForView(lib::tree::forview::Config),
    Barchart(lib::barchart::Config),
    Limitprinter(lib::limitprinter::Config),
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Str(s) => {
                if s.ends_with('\n') {
                    write!(f, "{}", s)
                } else {
                    writeln!(f, "{}", s)
                }
            }
            Output::String(s) => {
                if s.ends_with('\n') {
                    write!(f, "{}", s)
                } else {
                    writeln!(f, "{}", s)
                }
            }
            Output::TreeForSum(config) => write!(f, "{}", config.to_tree()),
            Output::TreeForView(config) => {
                if config.rl.is_empty() {
                    writeln!(f, "No transactions.")
                } else {
                    write!(f, "{}", config.to_tree())
                }
            }
            Output::Barchart(config) => {
                if config.rl.is_empty() {
                    writeln!(f, "No transactions.")
                } else {
                    write!(f, "{}", config.to_barchart())
                }
            }
            Output::Limitprinter(config) => write!(f, "{}", config.to_limitprinter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Output::Str("asdf"), "asdf\n")]
    #[case(Output::Str("asdf\n"), "asdf\n")]
    #[case(Output::String("asdf".into()), "asdf\n")]
    #[case(Output::String("asdf\n".into()), "asdf\n")]
    fn test_to_string(#[case] output: Output, #[case] want: impl Into<String>) {
        assert_eq!(output.to_string(), want.into())
    }
}
