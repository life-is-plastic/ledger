pub const INTERVAL_HELP: &str = "Interval of interest";
pub const INTERVAL_HELP_LONG: &str = "Interval of interest

Must be in the format 'A:B'. Each of 'A' or 'B' is either an ISO 8601 date (yyyy-mm-dd) or a relative date (see below). 'A' and 'B' are both optional, defaulting to 0000-01-01 and 9999-12-31 respectively.

A relative date is one of the following ('n' is optional and defaults to 0):
dn: n days from today
mn: first day of the nth month from today
Mn: last day of the nth month from today
yn: first day of the nth year from today
Yn: last day of the nth year from today

The following shorthands are also available:
dn = dn:dn
mn = mn:Mn
yn = yn:Yn";

#[derive(clap::Args)]
pub struct CategoriesOpts {
    /// Wildcard patterns to match categories of interest
    ///
    /// Use commas to separate multiple patterns. A transaction is included if
    /// its category matches any pattern.
    #[arg(
        short,
        long,
        value_name = "PATTERNS",
        value_delimiter = ',',
        default_value = "*"
    )]
    pub categories: Vec<String>,

    /// Wildcard patterns to match categories to exclude
    ///
    /// Use commas to separate multiple patterns. A transaction is excluded if
    /// its category matches any pattern. Takes precedence over '--categories'.
    #[arg(
        short = 'x',
        long,
        value_name = "PATTERNS",
        value_delimiter = ',',
        default_value = "",
        hide_default_value = true
    )]
    pub not_categories: Vec<String>,
}
