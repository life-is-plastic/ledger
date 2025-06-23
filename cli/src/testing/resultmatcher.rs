use crate::output::Output;

/// Helps check if a command returns the expected [`anyhow::Result<Output>`].
pub enum ResultMatcher<'a> {
    /// Asserts result is `Ok` and its payload equals the given value.
    OkExact(Output),

    /// Asserts result is an `Ok(Output::Str(_))` matching the given glob
    /// pattern. Matching is case-insensitive.
    OkStrGlob(&'a str),

    /// Asserts result is `Err` and that the error's `to_string()` matches the
    /// give glob pattern. Matching is case-insensitive.
    ErrGlob(&'a str),
}

impl ResultMatcher<'_> {
    pub fn assert_matches(&self, result: anyhow::Result<Output>) {
        match self {
            ResultMatcher::OkExact(want_output) => {
                if let Ok(got_output) = &result {
                    if got_output == want_output {
                        return;
                    }
                    text_diff::print_diff(
                        format!("{:?}", want_output).as_str(),
                        format!("{:?}", got_output).as_str(),
                        " ",
                    );
                    panic!("diff between want (red) and got (green), see above");
                }
                panic!("\n\twant: {:?}\n\tgot: {:?}\n", want_output, result);
            }
            ResultMatcher::OkStrGlob(pattern) => {
                let pattern_obj = wildmatch::WildMatch::new(pattern.to_lowercase().as_str());
                let matches = matches!(
                    result,
                    Ok(Output::Str(ref got_string)) if pattern_obj.matches(got_string.to_lowercase().as_str()),
                );
                assert!(
                    matches,
                    "\n\twant matches: Ok({:?})\n\tgot: {:?}\n",
                    pattern, result
                );
            }
            ResultMatcher::ErrGlob(pattern) => {
                let pattern_obj = wildmatch::WildMatch::new(pattern.to_lowercase().as_str());
                let matches = matches!(
                    result,
                    Err(ref got_err) if pattern_obj.matches(got_err.to_string().to_lowercase().as_str()),
                );
                assert!(
                    matches,
                    "\n\twant matches: Err({:?})\n\tgot: {:?}\n",
                    pattern, result
                );
            }
        }
    }
}
