use crate::tree::Node;
use crate::util;
use crate::Charset;
use crate::Record;
use crate::Recordlist;
use crate::Tree;

#[derive(Default)]
pub struct Config {
    pub charset: Charset,
    pub first_iid: usize,
    pub rl: Recordlist,
    /// Additional transformations to apply to a record's as-a-node string
    /// representation (records are leaf nodes). If not `None`, this is called
    /// once for each record in `rl`.
    #[allow(clippy::type_complexity)]
    pub leaf_string_postprocessor: Option<
        Box<
            dyn Fn(
                &Self,
                &Record,
                // Record's zero-based index-in-date.
                usize,
                // Record's original node string. Will be in one of these formats depending on
                // whether or not `r`'s note field is empty:
                //  {iid} -- {amount} {category}
                //  {iid} -- {amount} {category}: {note}
                String,
            ) -> String,
        >,
    >,
}

pub struct LsppArgs<'a> {
    pub r: &'a Record,
    /// `r`'s zero-based index-in-date.
    pub iid0: usize,
    /// `r`'s original node string. Will be in one of these formats depending on
    /// whether or not `r`'s note field is empty:
    /// 1. `{iid} -- {amount} {category}`
    /// 1. `{iid} -- {amount} {category}: {note}`
    pub leaf_string: String,
}

impl Config {
    pub fn to_tree(&self) -> Tree {
        let alignment_charlen = self.get_alignment_charlen();
        let mut root = Node::default();
        for (iid0, r) in self.rl.iter_with_iid() {
            self.make_year(&mut root, r, iid0, alignment_charlen);
        }
        Tree {
            charset: &self.charset,
            root,
        }
    }

    fn get_alignment_charlen(&self) -> usize {
        let char_count = |(iid0, r): (usize, &Record)| -> usize {
            util::count_digits((iid0 + self.first_iid) as u64)
                + util::BOUNDING_SPACES_COUNT
                + util::MIN_DASHES_COUNT
                + r.amount().charlen_for_alignment()
        };
        self.rl
            .iter_with_iid()
            .map(char_count)
            .max()
            .unwrap_or_default()
    }

    /// Constructs the string payload for the leaf node representing the given
    /// record.
    fn leaf_data(&self, r: &Record, iid0: usize, alignment_charlen: usize) -> String {
        let iid = iid0 + self.first_iid;
        let dash_count = alignment_charlen
            - util::count_digits(iid as u64)
            - util::BOUNDING_SPACES_COUNT
            - r.amount().charlen_for_alignment();
        let mut s = String::new();
        s.push_str(&iid.to_string());
        s.push(' ');
        for _ in 0..dash_count {
            s.push(self.charset.dash)
        }
        s.push(' ');
        s.push_str(&r.amount().to_string());
        // Give non-negative amounts a trailing space in order to vertically
        // align the beginning of `category`.
        if r.amount().0 >= 0 {
            s.push(' ');
        }
        s.push(' ');
        s.push_str(r.category().as_str());
        if !r.note().is_empty() {
            s.push_str(": ");
            s.push_str(r.note());
        }
        match self.leaf_string_postprocessor {
            Some(ref lspp) => lspp(self, r, iid0, s),
            None => s,
        }
    }

    fn make_leaf(&self, day: &mut Node, r: &Record, iid0: usize, alignment_charlen: usize) {
        let data = self.leaf_data(r, iid0, alignment_charlen);
        day.children.push(Node::new(data.into()));
    }

    fn make_day(&self, month: &mut Node, r: &Record, iid0: usize, alignment_charlen: usize) {
        #[rustfmt::skip]
        let strs = &[
            "",
            "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th",
            "11th", "12th", "13th", "14th", "15th", "16th", "17th", "18th", "19th", "20th",
            "21st", "22nd", "23rd", "24th", "25th", "26th", "27th", "28th", "29th", "30th",
            "31st",
        ];
        let s = strs[r.date().day() as usize];
        if !last_child_has_data(month, s) {
            month.children.push(Node::new(s.into()))
        }
        let day = last_child(month);
        self.make_leaf(day, r, iid0, alignment_charlen);
    }

    fn make_month(&self, year: &mut Node, r: &Record, iid0: usize, alignment_charlen: usize) {
        #[rustfmt::skip]
        let strs = &[
            "",
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let s = strs[r.date().month() as usize];
        if !last_child_has_data(year, s) {
            year.children.push(Node::new(s.into()))
        }
        let month = last_child(year);
        self.make_day(month, r, iid0, alignment_charlen);
    }

    fn make_year(&self, root: &mut Node, r: &Record, iid0: usize, alignment_charlen: usize) {
        let buf = [
            (r.date().year() / 1000) as u8 + b'0',
            (r.date().year() / 100 % 10) as u8 + b'0',
            (r.date().year() / 10 % 10) as u8 + b'0',
            (r.date().year() % 10) as u8 + b'0',
        ];
        let s = std::str::from_utf8(&buf).expect("all chars should be ascii");
        if !last_child_has_data(root, s) {
            root.children.push(Node::new(s.to_string().into()))
        }
        let year = last_child(root);
        self.make_month(year, r, iid0, alignment_charlen);
    }
}

fn last_child_has_data(node: &Node, data: &str) -> bool {
    match node.children.last() {
        Some(child) => child.data == data,
        None => false,
    }
}

fn last_child(node: &mut Node) -> &mut Node {
    node.children
        .last_mut()
        .expect("a node should have just been added")
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0, "", "")]
    #[case(
        100,
        r#"{"d":"0000-01-31","c":"aaa","a":0}"#,
        indoc!("
            0000
            `-- Jan
                `-- 31st
                    `-- 100 -- 0.00  aaa
        "),
    )]
    #[case(
        1,
        r#"
            {"d":"0000-01-31","c":"aaa","a":0}
            {"d":"2015-03-30","c":"b","a":1}
            {"d":"2015-03-30","c":"bb","a":1}
            {"d":"2015-03-30","c":"bbb","a":1}
            {"d":"2015-03-30","c":"bbbb","a":1}
            {"d":"2015-03-30","c":"bbbbb","a":1}
            {"d":"2015-03-30","c":"bbbbb","a":-1}
            {"d":"2015-03-30","c":"bbbb","a":-1}
            {"d":"2015-03-30","c":"bbb","a":-1}
            {"d":"2015-03-30","c":"bb","a":-1}
            {"d":"2015-03-30","c":"b","a":-123456789}
            {"d":"2015-03-31","c":"ccc","a":123456}
            {"d":"2015-04-01","c":"ddd","a":123456}
            {"d":"2015-04-02","c":"ddd","a":123456}
            {"d":"2015-04-03","c":"ddd","a":123456}
            {"d":"2015-04-04","c":"ddd","a":123456}
            {"d":"2015-05-10","c":"ddd","a":123456}
            {"d":"2015-05-11","c":"ddd","a":123456}
            {"d":"2015-05-12","c":"ddd","a":123456}
            {"d":"2015-05-13","c":"ddd","a":123456}
            {"d":"2015-05-14","c":"ddd","a":123456}
            {"d":"2015-05-20","c":"ddd","a":123456,"n":"some note"}
            {"d":"2015-05-21","c":"ddd","a":123456}
            {"d":"2015-05-22","c":"ddd","a":123456}
            {"d":"2015-05-23","c":"ddd","a":123456}
            {"d":"9999-10-24","c":"ddd","a":111}
        "#,
        indoc!("
            0000
            `-- Jan
                `-- 31st
                    `-- 1 ------------ 0.00  aaa
            2015
            |-- Mar
            |   |-- 30th
            |   |   |-- 1 ------------ 0.01  b
            |   |   |-- 2 ------------ 0.01  bb
            |   |   |-- 3 ------------ 0.01  bbb
            |   |   |-- 4 ------------ 0.01  bbbb
            |   |   |-- 5 ------------ 0.01  bbbbb
            |   |   |-- 6 ----------- (0.01) bbbbb
            |   |   |-- 7 ----------- (0.01) bbbb
            |   |   |-- 8 ----------- (0.01) bbb
            |   |   |-- 9 ----------- (0.01) bb
            |   |   `-- 10 -- (1,234,567.89) b
            |   `-- 31st
            |       `-- 1 -------- 1,234.56  ccc
            |-- Apr
            |   |-- 1st
            |   |   `-- 1 -------- 1,234.56  ddd
            |   |-- 2nd
            |   |   `-- 1 -------- 1,234.56  ddd
            |   |-- 3rd
            |   |   `-- 1 -------- 1,234.56  ddd
            |   `-- 4th
            |       `-- 1 -------- 1,234.56  ddd
            `-- May
                |-- 10th
                |   `-- 1 -------- 1,234.56  ddd
                |-- 11th
                |   `-- 1 -------- 1,234.56  ddd
                |-- 12th
                |   `-- 1 -------- 1,234.56  ddd
                |-- 13th
                |   `-- 1 -------- 1,234.56  ddd
                |-- 14th
                |   `-- 1 -------- 1,234.56  ddd
                |-- 20th
                |   `-- 1 -------- 1,234.56  ddd: some note
                |-- 21st
                |   `-- 1 -------- 1,234.56  ddd
                |-- 22nd
                |   `-- 1 -------- 1,234.56  ddd
                `-- 23rd
                    `-- 1 -------- 1,234.56  ddd
            9999
            `-- Oct
                `-- 24th
                    `-- 1 ------------ 1.11  ddd
        "),
    )]
    fn test_into_tree(#[case] first_iid: usize, #[case] rl: Recordlist, #[case] want: &str) {
        let config = Config {
            charset: Charset::default(),
            first_iid,
            leaf_string_postprocessor: None,
            rl,
        };
        let tr = config.to_tree();
        assert_eq!(tr.to_string(), want);
    }
}
