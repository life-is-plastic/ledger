use crate::base;
use crate::cli;

/// A single command invocation.
pub struct Invocation<'a> {
    /// Command line arguments. First arg is the binary name, which doesn't do
    /// anything useful, so can be empty.
    pub args: &'a [&'a str],
    pub res: cli::testing::ResultMatcher<'a>,
}

/// Test case encapsulating expectations for the given command invocations.
/// Commands may mutate the filesystem.
pub struct MutCase<'a> {
    pub invocations: &'a [Invocation<'a>],

    /// Filesystem state prior to running the command.
    pub initial_state: cli::testing::StrState<'a>,

    /// Desired filesystem state after running the command.
    pub final_state: cli::testing::State,
}

impl MutCase<'_> {
    /// 1. Creates a tempdir and initializes files based on `initial_state`
    /// 1. Runs each command and checks result using `matcher`
    /// 1. Checks if files match `final_state`
    pub fn run(self) {
        let td = tempfile::TempDir::new().unwrap();
        let fs = base::Fs::new(td.path());
        self.initial_state.to_fs(&fs);

        for inv in self.invocations {
            let root = match <cli::Root as clap::Parser>::try_parse_from(inv.args) {
                Ok(cmd) => cmd,
                Err(e) => panic!("{}", e),
            };
            let res = root.run(&fs);
            inv.res.assert_matches(res);
        }

        let got_final_state = cli::testing::State::from_fs(&fs);
        let want_final_state = self.final_state.into();
        assert_eq!(got_final_state, want_final_state);
    }
}

/// Test case encapsulating expectations for the given command invocations.
/// Commands are expected to leave filesystem state unchanged.
pub struct Case<'a> {
    pub invocations: &'a [Invocation<'a>],
    pub initial_state: cli::testing::StrState<'a>,
}

impl Case<'_> {
    /// 1. Creates a tempdir and initializes files based on `initial_state`
    /// 1. Runs each command and checks result using `matcher`
    /// 1. Checks if files match `initial_state`
    pub fn run(self) {
        let tc = MutCase {
            invocations: self.invocations,
            final_state: self.initial_state.to_state(),
            initial_state: self.initial_state,
        };
        tc.run()
    }
}

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
