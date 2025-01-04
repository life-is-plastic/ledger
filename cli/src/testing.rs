mod case;
mod resultmatcher;
mod state;

pub use case::Case;
pub use case::Invocation;
pub use case::MutCase;
pub use resultmatcher::ResultMatcher;
pub use state::State;
pub use state::StrState;

/// Generates test functions from test cases.
///
/// Accepts one or more tuples of the form `(testcase_name: ident, testcase:
/// Case|MutCase)`. Creates a submodule named `cmd_testcases` in the caller's
/// module, and then for each test case tuple, creates a corresponding function
/// named `testcase_name`.
macro_rules! generate_testcases {
    ($(($name:ident, $testcase:expr)),+ $(,)?) => {
        mod cmd_testcases {
            use super::*;

            $(
                #[test]
                fn $name() {
                    $testcase.run()
                }
            )+
        }
    };
}
pub(crate) use generate_testcases;

/// Returns a filesystem object anchored at a temporary directory. The `Fs` must
/// not outlive the returned `TempDir`.
pub fn tempfs() -> (lib::Fs, tempfile::TempDir) {
    let td = tempfile::TempDir::new().unwrap();
    let fs = lib::Fs::new(td.path());
    (fs, td)
}
