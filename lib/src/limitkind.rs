use crate::Cents;
use crate::Date;
use crate::Limits;
use crate::Recordlist;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::Display,
    strum::AsRefStr,
    strum::IntoStaticStr,
    strum::EnumVariantNames,
)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Limitkind {
    Rrsp,
    Tfsa,
}

impl Limitkind {
    pub fn remaining(self, limits: &Limits, rl: &Recordlist, today: Date) -> Cents {
        match self {
            Limitkind::Rrsp => Self::remaining_rrsp(limits, rl, today),
            Limitkind::Tfsa => Self::remaining_tfsa(limits, rl, today),
        }
    }

    fn remaining_rrsp(limits: &Limits, rl: &Recordlist, today: Date) -> Cents {
        let contributions = rl
            .iter()
            .map(
                |r| match r.date().year() <= today.year() && r.amount().0 > 0 {
                    true => r.amount().0,
                    false => 0,
                },
            )
            .sum::<i64>();
        Cents(limits.inception_to_year(today.year()) - contributions)
    }

    fn remaining_tfsa(limits: &Limits, rl: &Recordlist, today: Date) -> Cents {
        let contributions = rl
            .iter()
            .map(
                |r| match r.date().year() <= today.year() && r.amount().0 > 0 {
                    true => r.amount().0,
                    false => 0,
                },
            )
            .sum::<i64>();
        let withdrawals_before_year = rl
            .iter()
            .map(
                |r| match r.date().year() < today.year() && r.amount().0 < 0 {
                    true => -r.amount().0,
                    false => 0,
                },
            )
            .sum::<i64>();
        Cents(limits.inception_to_year(today.year()) - contributions + withdrawals_before_year)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("{}", "", "d", Cents(0), Cents(0))]
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
        "2014-01-01",
        Cents(-1000 - 500),
        Cents(-1000 - 500),
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
        "2015-01-01",
        Cents(5000 - 1000 - 500 - 2000),
        Cents(5000 - 1000 - 500 - 2000),
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
        "2016-01-01",
        Cents(5000 + 5000 - 1000 - 500 - 2000 - 3000),
        Cents(5000 + 5000 - 1000 - 500 - 2000 + 10000 - 3000),
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
        "2017-01-01",
        Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 - 3000 - 10000),
        Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 + 10000 - 3000 - 10000),
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
        "2018-01-01",
        Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 - 3000 - 10000 - 4000),
        Cents(5000 + 5000 + 5000 - 1000 - 500 - 2000 + 10000 - 3000 - 10000 - 4000),
    )]
    fn test_remaining(
        #[case] limits: Limits,
        #[case] rl: Recordlist,
        #[case] today: Date,
        #[case] want_rrsp: Cents,
        #[case] want_tfsa: Cents,
    ) {
        assert_eq!(Limitkind::Rrsp.remaining(&limits, &rl, today,), want_rrsp);
        assert_eq!(Limitkind::Tfsa.remaining(&limits, &rl, today,), want_tfsa);
    }
}
