#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "&str")]
pub struct Category(String);

impl Category {
    pub const SEP: &str = "/";
    pub const LEVEL0: &str = "All";

    pub fn str(&self) -> &str {
        &self.0
    }

    pub fn level(&self, level: usize) -> &str {
        if level == 0 {
            return Self::LEVEL0;
        }
        match self
            .0
            .match_indices(Self::SEP)
            .map(|(i, _)| i)
            .nth(level - 1)
        {
            Some(i) => &self.str()[..i],
            None => self.str(),
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.str().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("input is empty")]
    Empty,
    #[error("input starts or ends with '{}'", Category::SEP)]
    TerminalSeparator,
    #[error("input contains consecutive occurrences of '{}'", Category::SEP)]
    ConsecutiveSeparators,
}

impl ParseError {
    fn check(s: &str) -> Result<(), Self> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        if s.starts_with(Category::SEP) || s.ends_with(Category::SEP) {
            return Err(ParseError::TerminalSeparator);
        }
        if s.contains(&(Category::SEP.to_string() + Category::SEP)) {
            return Err(ParseError::ConsecutiveSeparators);
        }
        Ok(())
    }
}

impl std::str::FromStr for Category {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::Err::check(s)?;
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<&str> for Category {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", false)]
    #[case("/", false)]
    #[case("//", false)]
    #[case("/asdf", false)]
    #[case("asdf/", false)]
    #[case("as//df", false)]
    #[case("asdf", true)]
    #[case("as/df", true)]
    #[case("a/sd/f", true)]
    #[case("a", true)]
    fn test_from_str(#[case] s: &str, #[case] is_ok: bool) {
        assert_eq!(s.parse::<Category>().is_ok(), is_ok)
    }

    #[rstest]
    #[case("a", 0, "All")]
    #[case("a", 1, "a")]
    #[case("a", 2, "a")]
    #[case("a/b/c/d", 0, "All")]
    #[case("a/b/c/d", 1, "a")]
    #[case("a/b/c/d", 2, "a/b")]
    #[case("a/b/c/d", 3, "a/b/c")]
    #[case("a/b/c/d", 4, "a/b/c/d")]
    #[case("a/b/c/d", 5, "a/b/c/d")]
    #[case("a/b/c/d", 100, "a/b/c/d")]
    fn test_level(#[case] cat: Category, #[case] level: usize, #[case] want: &str) {
        assert_eq!(cat.level(level), want);
    }
}
