use crate::util;
use crate::Aggregate;
use crate::Cents;
use crate::Charset;
use crate::Date;
use crate::Datepart;
use crate::Interval;
use crate::Recordlist;

pub struct Barchart<'cs> {
    charset: &'cs Charset,
    bounds: Interval,
    unit: Datepart,
    pos: Aggregate<Date, Cents>,
    neg: Aggregate<Date, Cents>, // absolute value
    label_charlen: usize,
    max_val: Cents,
    max_barlen: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub charset: Charset,
    pub bounds: Interval,
    pub unit: Datepart,
    pub term_width: usize,
    pub rl: Recordlist,
}

impl Config {
    pub fn to_barchart(&self) -> Barchart {
        let bounds = self.rl.spanned_interval().intersection(self.bounds);
        let mut pos = Aggregate::<Date, Cents>::default();
        let mut neg = Aggregate::<Date, Cents>::default();
        for interval in bounds.iter(self.unit) {
            for r in self.rl.slice_spanning_interval(interval) {
                match r.amount().cmp(&Cents(0)) {
                    std::cmp::Ordering::Greater => pos.add(interval.start, r.amount()),
                    std::cmp::Ordering::Less => neg.add(interval.start, -r.amount()),
                    _ => {}
                }
            }
        }

        let label_charlen = match self.unit {
            Datepart::Year => 4,  // yyyy
            Datepart::Month => 8, // yyyy mmm
            Datepart::Day => 10,  // yyyy-mm-dd
        };
        let max_val = Cents::max(
            pos.iter().map(|(_, v)| v).max().unwrap_or_default(),
            neg.iter().map(|(_, v)| v).max().unwrap_or_default(),
        );
        let max_barlen = self.term_width.max(util::MIN_TERM_WIDTH)
            - label_charlen // max 10
            - util::BOUNDING_SPACES_COUNT
            - 1 // vertical divider just before bar
            - (-max_val).charlen(); // max 27

        Barchart {
            charset: &self.charset,
            bounds,
            unit: self.unit,
            pos,
            neg,
            label_charlen,
            max_val,
            max_barlen,
        }
    }
}

impl Barchart<'_> {
    pub fn is_empty(&self) -> bool {
        self.bounds.is_empty()
    }

    fn label(&self, dt: Date) -> String {
        let fmt = match self.unit {
            Datepart::Year => time::macros::format_description!("[year]"),
            Datepart::Month => time::macros::format_description!("[year] [month repr:short]"),
            Datepart::Day => time::macros::format_description!("[year]-[month]-[day]"),
        };
        dt.format(fmt).expect("formatting should succeed")
    }

    fn barlen(&self, val: Cents) -> usize {
        let x = (val.0 as f64) / (self.max_val.0 as f64) * (self.max_barlen as f64);
        self.max_barlen.min(x.round() as usize)
    }

    fn draw(&self, w: &mut impl std::fmt::Write, dt: Date) -> std::fmt::Result {
        if self.pos.is_empty() && self.neg.is_empty() {
            return Ok(());
        }
        write!(w, "{} |", self.label(dt))?;
        if !self.pos.is_empty() {
            let val = self.pos.get(dt).unwrap_or_default();
            let barlen = self.barlen(val);
            if barlen > 0 {
                w.write_str(self.charset.color_prefix_green)?;
                for _ in 0..barlen {
                    w.write_char(self.charset.chart_bar_pos)?;
                }
                w.write_str(self.charset.color_suffix)?;
                w.write_char(' ')?;
            }
            writeln!(w, "{}", val)?;

            if self.neg.is_empty() {
                return Ok(());
            }
            for _ in 0..(self.label_charlen) {
                w.write_char(' ')?;
            }
            w.write_str(" |")?;
        }
        let val = self.neg.get(dt).unwrap_or_default();
        let barlen = self.barlen(val);
        if barlen > 0 {
            w.write_str(self.charset.color_prefix_red)?;
            for _ in 0..barlen {
                w.write_char(self.charset.chart_bar_neg)?;
            }
            w.write_str(self.charset.color_suffix)?;
            w.write_char(' ')?;
        }
        writeln!(w, "{}", -val)?;
        Ok(())
    }
}

impl std::fmt::Display for Barchart<'_> {
    /// Writes a terminating newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for interval in self.bounds.iter(self.unit) {
            self.draw(f, interval.start)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Charset;
    use crate::Datepart;
    use crate::Interval;
    use indoc::indoc;
    use rstest::fixture;
    use rstest::rstest;

    #[fixture]
    fn rl() -> Recordlist {
        r#"
            {"d":"2015-03-30","c":"aaa","a":10000}
            {"d":"2015-03-30","c":"aaa","a":5000}
            {"d":"2015-03-30","c":"aaa","a":-5000}
            {"d":"2015-03-30","c":"aaa","a":-2000}
            {"d":"2015-03-31","c":"aaa","a":2000}
            {"d":"2015-04-29","c":"aaa","a":-2000}
            {"d":"2015-05-02","c":"aaa","a":-2000}
            {"d":"2015-05-05","c":"aaa","a":2000}
            {"d":"2015-05-06","c":"aaa","a":2000}
        "#
        .parse()
        .unwrap()
    }

    #[rstest]
    #[case("0000-01-01:2010-12-31", "day", "")]
    #[case("2015-03-30", "day", indoc!("
        2015-03-30 |+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 150.00
                   |---------------------------- (70.00)
    "))]
    #[case("2015-03-30", "month", indoc!("
        2015 Mar |+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 150.00
                 |---------------------------- (70.00)
    "))]
    #[case("2015-03-30", "year", indoc!("
        2015 |+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 150.00
             |------------------------------ (70.00)
    "))]
    #[case("2015-04-29:2015-05-02", "day", indoc!("
        2015-04-29 |------------------------------------------------------------ (20.00)
        2015-04-30 |0.00
        2015-05-01 |0.00
        2015-05-02 |------------------------------------------------------------ (20.00)
    "))]
    #[case("2015-05-05:2015-05-06", "day", indoc!("
        2015-05-05 |++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 20.00
        2015-05-06 |++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 20.00
    "))]
    #[case(":", "month", indoc!("
        2015 Mar |+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++ 170.00
                 |------------------------- (70.00)
        2015 Apr |0.00
                 |------- (20.00)
        2015 May |++++++++++++++ 40.00
                 |------- (20.00)
    "))]
    fn test_barchart(
        rl: Recordlist,
        #[case] bounds: Interval,
        #[case] unit: Datepart,
        #[case] want: &str,
    ) {
        let config = Config {
            charset: Charset::default(),
            bounds,
            unit,
            rl,
            term_width: 80,
        };
        let chart = config.to_barchart();
        assert_eq!(chart.to_string(), want)
    }
}
