use crate::cents::Cents;
use crate::charset::Charset;
use crate::limitkind::Limitkind;
use crate::limits::Limits;
use crate::recordlist::Recordlist;
use crate::util;

pub struct Limitprinter<'a> {
    charset: &'a Charset,
    /// Sorted yearly limits.
    limits: Vec<(String, Cents)>,
    /// Total limit and remaining limit, in that order.
    summary: [(String, Cents); 2],
    alignment_charlen: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub charset: Charset,
    pub year: u16,
    pub kind: Limitkind,
    pub limits: Limits,
    pub rl: Recordlist,
}

impl Config {
    pub fn to_limitprinter(&self) -> Limitprinter {
        let limits = self
            .limits
            .range(..=self.year)
            .map(|(year, limit)| (format!("{:0>4}", year), limit))
            .collect::<Vec<_>>();

        let total = limits.iter().map(|&(_, limit)| limit).sum::<Cents>();
        let remaining = self.kind.remaining(&self.limits, &self.rl, self.year);
        let summary = [("Total".into(), total), ("Remaining".into(), remaining)];

        fn char_count((label, value): &(String, Cents)) -> usize {
            label.len()
                + util::BOUNDING_SPACES_COUNT
                + util::MIN_DASHES_COUNT
                + value.charlen_for_alignment()
        }
        let alignment_charlen = usize::max(
            limits.iter().map(char_count).max().unwrap_or_default(),
            summary.iter().map(char_count).max().unwrap_or_default(),
        );

        Limitprinter {
            charset: &self.charset,
            limits,
            summary,
            alignment_charlen,
        }
    }
}

impl Limitprinter<'_> {
    fn draw(
        &self,
        w: &mut impl std::fmt::Write,
        (label, value): &(String, Cents),
    ) -> std::fmt::Result {
        let dash_count = self.alignment_charlen
            - label.len()
            - util::BOUNDING_SPACES_COUNT
            - value.charlen_for_alignment();
        w.write_str(label)?;
        w.write_char(' ')?;
        for _ in 0..dash_count {
            w.write_char(self.charset.dash)?;
        }
        w.write_char(' ')?;
        writeln!(w, "{}", value)?;
        Ok(())
    }
}

impl std::fmt::Display for Limitprinter<'_> {
    /// Writes a terminating newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.limits.iter().try_for_each(|row| self.draw(f, row))?;
        if !self.limits.is_empty() {
            use std::fmt::Write;
            for _ in 1..self.alignment_charlen {
                f.write_char('=')?;
            }
            f.write_char('\n')?;
        }
        self.summary.iter().try_for_each(|row| self.draw(f, row))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use rstest::rstest;

    #[rstest]
    #[case(
        2015,
        Limitkind::Rrsp,
        "{}",
        "",
        indoc!("
            Total ------ 0.00
            Remaining -- 0.00
        ")
    )]
    #[case(
        2015,
        Limitkind::Rrsp,
        "{}",
        r#"{"d":"2015-03-30","c":"aaa","a":100000}"#,
        indoc!("
            Total ----------- 0.00
            Remaining -- (1,000.00)
        ")
    )]
    #[case(
        2015,
        Limitkind::Tfsa,
        "{}",
        r#"{"d":"2014-03-30","c":"aaa","a":-100000}"#,
        indoc!("
            Total ---------- 0.00
            Remaining -- 1,000.00
        ")
    )]
    #[case(
        2015,
        Limitkind::Rrsp,
        r#"{
            "40": 100000,
            "2013": 200000,
            "2014": 50000000
        }"#,
        r#"{"d":"0035-03-30","c":"aaa","a":50300500}"#,
        indoc!("
            0040 ----- 1,000.00
            2013 ----- 2,000.00
            2014 --- 500,000.00
            ===================
            Total -- 503,000.00
            Remaining --- (5.00)
        ")
    )]
    fn test_to_string(
        #[case] year: u16,
        #[case] kind: Limitkind,
        #[case] limits: Limits,
        #[case] rl: Recordlist,
        #[case] want: &str,
    ) {
        let config = Config {
            charset: Charset::default(),
            year,
            kind,
            limits,
            rl,
        };
        let printer = config.to_limitprinter();
        assert_eq!(printer.to_string(), want)
    }
}
