use crate::testing::state;
use crate::testing::ResultMatcher;

/// A single command invocation.
pub struct Invocation<'a> {
    /// Command line arguments. First arg is the binary name, which doesn't do
    /// anything useful, so can be empty.
    pub args: &'a [&'a str],
    pub res: ResultMatcher<'a>,
}

/// Test case encapsulating expectations for the given command invocations.
/// Commands may mutate the filesystem.
pub struct MutCase<'a> {
    pub invocations: &'a [Invocation<'a>],

    /// Filesystem state prior to running the command.
    pub initial_state: state::StrState<'a>,

    /// Desired filesystem state after running the command.
    pub final_state: state::State,
}

impl MutCase<'_> {
    /// 1. Creates a tempdir and initializes files based on `initial_state`
    /// 1. Runs each command and checks result using `matcher`
    /// 1. Checks if files match `final_state`
    pub fn run(self) {
        let td = tempfile::TempDir::new().unwrap();
        let fs = lib::Fs::new(td.path());
        self.initial_state.to_fs(&fs);

        for inv in self.invocations {
            let root = match <crate::cmds::Root as clap::Parser>::try_parse_from(inv.args) {
                Ok(cmd) => cmd,
                Err(e) => panic!("{}", e),
            };
            let res = root.run(&fs);
            inv.res.assert_matches(res);
        }

        let got_final_state = state::State::from_fs(&fs);
        let want_final_state = self.final_state.into();
        assert_eq!(got_final_state, want_final_state);
    }
}

/// Test case encapsulating expectations for the given command invocations.
/// Commands are expected to leave filesystem state unchanged.
pub struct Case<'a> {
    pub invocations: &'a [Invocation<'a>],
    pub initial_state: state::StrState<'a>,
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
