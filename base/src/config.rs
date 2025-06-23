use crate::category::Category;
use crate::cents::Cents;
use crate::limitkind::Limitkind;

/// Application config.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Config {
    pub first_index_in_date: usize,
    pub lim_account_type: Option<Limitkind>,
    pub unsigned_is_negative: bool,
    pub use_colored_output: bool,
    pub use_unicode_symbols: bool,
    pub templates: std::collections::HashMap<String, Vec<TemplateEntry>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateEntry {
    pub category: Category,
    pub amount: Cents,
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

impl TryFrom<&str> for Config {
    type Error = <Self as std::str::FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<Self>()
    }
}
