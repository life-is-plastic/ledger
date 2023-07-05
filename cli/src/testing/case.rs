use crate::testing::state;
use crate::testing::ResultMatcher;

/// Test case encapsulating expectations for a single command invocation. The
/// command is expected to leave filesystem state unchanged.
pub struct Case<'a> {
    /// Command line arguments. First arg is the binary name, which doesn't do
    /// anything useful, so can be empty.
    pub args: &'a [&'a str],

    pub matcher: ResultMatcher<'a>,

    /// Filesystem state prior to running the command.
    pub initial_state: state::StrState<'a>,
}

impl Case<'_> {
    /// 1. Creates a tempdir and initializes files based on `initial_state`
    /// 1. Runs command and checks result using `matcher`
    /// 1. Checks if files match `initial_state`
    pub fn run(self) {
        let tc = MutCase {
            args: self.args,
            matcher: self.matcher,
            final_state: self.initial_state.to_state(),
            initial_state: self.initial_state,
        };
        tc.run()
    }
}

/// Test case encapsulating expectations for a single command invocation. The
/// command may mutate the filesystem.
pub struct MutCase<'a> {
    /// Same usage as in [`Case`].
    pub args: &'a [&'a str],

    pub matcher: ResultMatcher<'a>,

    /// Same usage as in [`Case`].
    pub initial_state: state::StrState<'a>,

    /// Desired filesystem state after running the command.
    pub final_state: state::State,
}

impl MutCase<'_> {
    /// 1. Creates a tempdir and initializes files based on `initial_state`
    /// 1. Runs command and checks result using `matcher`
    /// 1. Checks if files match `final_state`
    pub fn run(self) {
        let td = tempfile::TempDir::new().unwrap();
        let fs = lib::Fs::new(td.path());
        self.initial_state.to_fs(&fs);

        let root = match <crate::cmds::Root as clap::Parser>::try_parse_from(self.args) {
            Ok(cmd) => cmd,
            Err(e) => panic!("{}", e),
        };
        let res = root.run(&fs);
        self.matcher.assert_matches(res);

        let got_final_state = state::State::from_fs(&fs);
        let want_final_state = self.final_state.into();
        assert_eq!(got_final_state, want_final_state);
    }
}
