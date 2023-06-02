/// Application config.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub first_index_in_date: usize,
    pub lim_account_type: String,
    pub unsigned_is_positive: bool,
    pub use_colored_output: bool,
    pub use_unicode_symbols: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            first_index_in_date: 1,
            lim_account_type: String::new(),
            unsigned_is_positive: true,
            use_colored_output: true,
            use_unicode_symbols: true,
        }
    }
}

impl std::fmt::Display for Config {
    /// Writes a terminating newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
        writeln!(f, "{}", s)
    }
}

impl std::str::FromStr for Config {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
