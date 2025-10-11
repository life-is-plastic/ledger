use crate::base;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Limitkind {
    Rrsp,
    Tfsa,
}

impl Limitkind {
    pub fn remaining(self, limits: &base::Limits, rl: &base::Recordlist, year: u16) -> base::Cents {
        match self {
            Limitkind::Rrsp => Self::remaining_rrsp(limits, rl, year),
            Limitkind::Tfsa => Self::remaining_tfsa(limits, rl, year),
        }
    }

    fn remaining_rrsp(limits: &base::Limits, rl: &base::Recordlist, year: u16) -> base::Cents {
        let contributions = rl
            .iter()
            .map(|r| match r.date().year() <= year && r.amount().0 > 0 {
                true => r.amount(),
                false => base::Cents(0),
            })
            .sum();
        limits.inception_to_year(year) - contributions
    }

    fn remaining_tfsa(limits: &base::Limits, rl: &base::Recordlist, year: u16) -> base::Cents {
        let contributions = rl
            .iter()
            .map(|r| match r.date().year() <= year && r.amount().0 > 0 {
                true => r.amount(),
                false => base::Cents(0),
            })
            .sum();
        let withdrawals_before_year = rl
            .iter()
            .map(|r| match r.date().year() < year && r.amount().0 < 0 {
                true => -r.amount(),
                false => base::Cents(0),
            })
            .sum();
        limits.inception_to_year(year) - contributions + withdrawals_before_year
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("{}", "", 2015, base::Cents(0), base::Cents(0))]
    #[case(
        r#"{
            "2015": 5000,
            "2016": 5000,
            "2017": 5000
        }"#,
        r#"
            {"d":"2014-01-01","c":"aaa","a":1000}
            {"d":"2014-01-01","c":"aaa","a":500}
            {"d":"2015-01-01","c":"aaa","a":2000}
            {"d":"2015-01-01","c":"aaa","a":-10000}
            {"d":"2016-01-01","c":"aaa","a":3000}
            {"d":"2017-01-01","c":"aaa","a":10000}
            {"d":"2018-01-01","c":"aaa","a":4000}
            {"d":"2018-01-01","c":"aaa","a":4000}
        "#,
        2014,
        base::Cents(-1000 - 500),
        base::Cents(-1000 - 500),
    )]
    #[case(
        r#"{
            "2015": 5000,
            "2016": 5000,
            "2017": 5000
        }"#,
        r#"
            {"d":"2014-01-01","c":"aaa","a":1000}
            {"d":"2014-01-01","c":"aaa","a":500}
            {"d":"2015-01-01","c":"aaa","a":2000}
            {"d":"2015-01-01","c":"aaa","a":-10000}
            {"d":"2016-01-01","c":"aaa","a":3000}
            {"d":"2017-01-01","c":"aaa","a":10000}
            {"d":"2018-01-01","c":"aaa","a":4000}
        "#,
        2015,
        base::Cents(5000 - 1000 - 500 - 2000),
        base::Cents(5000 - 1000 - 500 - 2000),
    )]
    #[case(
        r#"{
            "2015": 5000,
            "2016": 5000,
            "2017": 5000
        }"#,
        r#"
            {"d":"2014-01-01","c":"aaa","a":1000}
            {"d":"2014-01-01","c":"aaa","a":500}
            {"d":"2015-01-01","c":"aaa","a":2000}
            {"d":"2015-01-01","c":"aaa","a":-10000}
            {"d":"2016-01-01","c":"aaa","a":3000}
            {"d":"2017-01-01","c":"aaa","a":10000}
            {"d":"2018-01-01","c":"aaa","a":4000}
        "#,
        2016,
        base::Cents(5000 + 5000 - 1000 - 500 - 2000 - 3000),
        base::Cents(5000 + 5000 - 1000 - 500 - 2000 + 10000 - 3000),
    )]
    #[case(
        r#"{
            "2015": 5000,
            "2016": 5000,
            "2017": 5000
        }"#,
        r#"
            {"d":"2014-01-01","c":"aaa","a":1000}
            {"d":"2014-01-01","c":"aaa","a":500}
            {"d":"2015-01-01","c":"aaa","a":2000}
            {"d":"2015-01-01","c":"aaa","a":-10000}
            {"d":"2016-01-01","c":"aaa","a":3000}
            {"d":"2017-01-01","c":"aaa","a":10000}
            {"d":"2018-01-01","c":"aaa","a":4000}
        "#,
        2017,
        base::Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 - 3000 - 10000),
        base::Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 + 10000 - 3000 - 10000),
    )]
    #[case(
        r#"{
            "2015": 5000,
            "2016": 5000,
            "2017": 5000
        }"#,
        r#"
            {"d":"2014-01-01","c":"aaa","a":1000}
            {"d":"2014-01-01","c":"aaa","a":500}
            {"d":"2015-01-01","c":"aaa","a":2000}
            {"d":"2015-01-01","c":"aaa","a":-10000}
            {"d":"2016-01-01","c":"aaa","a":3000}
            {"d":"2017-01-01","c":"aaa","a":10000}
            {"d":"2018-01-01","c":"aaa","a":4000}
        "#,
        2018,
        base::Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 - 3000 - 10000 - 4000),
        base::Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 + 10000 - 3000 - 10000 - 4000),
    )]
    fn test_remaining(
        #[case] limits: base::Limits,
        #[case] rl: base::Recordlist,
        #[case] current_year: u16,
        #[case] want_rrsp: base::Cents,
        #[case] want_tfsa: base::Cents,
    ) {
        assert_eq!(
            Limitkind::Rrsp.remaining(&limits, &rl, current_year),
            want_rrsp
        );
        assert_eq!(
            Limitkind::Tfsa.remaining(&limits, &rl, current_year),
            want_tfsa
        );
    }
}
