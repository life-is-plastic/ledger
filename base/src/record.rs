use crate::category::Category;
use crate::cents::Cents;
use crate::date::Date;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(rename = "d")]
    date: Date,
    #[serde(rename = "c")]
    category: Category,
    #[serde(rename = "a")]
    amount: Cents,
    #[serde(rename = "n", skip_serializing_if = "String::is_empty", default)]
    note: String,
}

impl Record {
    pub fn date(&self) -> Date {
        self.date
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn amount(&self) -> Cents {
        self.amount
    }

    pub fn note(&self) -> &str {
        &self.note
    }

    pub fn new(date: Date, category: Category, amount: Cents, note: String) -> Self {
        Self {
            date,
            category,
            amount,
            note,
        }
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        f.write_str(&s)
    }
}

impl std::str::FromStr for Record {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl TryFrom<&str> for Record {
    type Error = <Self as std::str::FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        r#"{"d":"0000-01-01","c":"category","a":123456}"#,
        Record {
            date: Date::MIN,
            category: "category".parse().unwrap(),
            amount: Cents(123456),
            note: String::new(),
        },
    )]
    #[case(
        r#"{"d":"9999-12-31","c":"category","a":0,"n":"some note\nmore note"}"#,
        Record {
            date: Date::MAX,
            category: "category".parse().unwrap(),
            amount: Cents(-0),
            note: String::from("some note\nmore note"),
        },
    )]
    fn test_serde(#[case] s: &str, #[case] r: Record) {
        assert_eq!(s.parse::<Record>().unwrap(), r);
        assert_eq!(r.to_string(), s);
    }

    #[rstest]
    #[case(r#"{"d":"m","c":"category","a":123456}"#)]
    #[case(r#"{"d":"2015-03-30","c":"","a":123456}"#)]
    #[case(r#"{"d":"2015-03-30","c":"/category","a":123456}"#)]
    #[case(r#"{"d":"2015-03-30","c":"category","a":1234.56}"#)]
    #[case(r#"{"d":"2015-03-30","c":"category","a":123456,"unknown_field":""}"#)]
    fn test_deserialize_failing(#[case] s: &str) {
        assert!(s.parse::<Record>().is_err())
    }
}
