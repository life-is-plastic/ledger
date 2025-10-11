use crate::base;

pub struct Config {
    pub charset: base::Charset,
    pub first_iid: usize,
    pub rl: base::Recordlist,
    /// Additional transformations to apply to a record's as-a-node string
    /// representation (records are leaf nodes). If not `None`, this is called
    /// once for each record in `rl`.
    #[allow(clippy::type_complexity)]
    pub leaf_string_postprocessor: Option<
        Box<
            dyn Fn(
                &Self,
                &base::Record,
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

impl Eq for Config {}
impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.charset == other.charset
            && self.first_iid == other.first_iid
            && self.rl == other.rl
            // This is a simplification. In general, the only way to tell if two
            // functions are equal is to check if both produce equal outputs for
            // all inputs, which is not feasible.
            && self.leaf_string_postprocessor.is_some()
                == other.leaf_string_postprocessor.is_some()
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("charset", &self.charset)
            .field("first_iid", &self.first_iid)
            .field("rl", &self.rl)
            .field(
                "leaf_string_postprocessor",
                &self
                    .leaf_string_postprocessor
                    .as_ref()
                    .map(|b| b as *const _),
            )
            .finish()
    }
}

impl Config {
    pub fn to_tree(&'_ self) -> base::Tree<'_> {
        let alignment_charlen = self.get_alignment_charlen();
        let mut root = base::tree::Node::default();
        for (iid0, r) in self.rl.iter_with_iid() {
            self.make_year_node(&mut root, r, iid0, alignment_charlen);
        }
        base::Tree {
            charset: &self.charset,
            root,
        }
    }

    fn get_alignment_charlen(&self) -> usize {
        let char_count = |(iid0, r): (usize, &base::Record)| -> usize {
            base::util::count_digits((iid0 + self.first_iid) as u64)
                + base::util::BOUNDING_SPACES_COUNT
                + base::util::MIN_DASHES_COUNT
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
    fn leaf_data(&self, r: &base::Record, iid0: usize, alignment_charlen: usize) -> String {
        let iid = iid0 + self.first_iid;
        let dash_count = alignment_charlen
            - base::util::count_digits(iid as u64)
            - base::util::BOUNDING_SPACES_COUNT
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

    fn make_leaf_node(
        &self,
        day: &mut base::tree::Node,
        r: &base::Record,
        iid0: usize,
        alignment_charlen: usize,
    ) {
        let data = self.leaf_data(r, iid0, alignment_charlen);
        day.children.push(base::tree::Node::new(data.into()));
    }

    fn make_day_node(
        &self,
        month: &mut base::tree::Node,
        r: &base::Record,
        iid0: usize,
        alignment_charlen: usize,
    ) {
        #[rustfmt::skip]
        let strs = &[
            "",
            "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th",
            "11th", "12th", "13th", "14th", "15th", "16th", "17th", "18th", "19th", "20th",
            "21st", "22nd", "23rd", "24th", "25th", "26th", "27th", "28th", "29th", "30th",
            "31st",
        ];
        let s = strs[r.date().day() as usize];
        if !last_child_exists_and_has_expected_data(month, s) {
            month.children.push(base::tree::Node::new(s.into()))
        }
        let day = last_child(month);
        self.make_leaf_node(day, r, iid0, alignment_charlen);
    }

    fn make_month_node(
        &self,
        year: &mut base::tree::Node,
        r: &base::Record,
        iid0: usize,
        alignment_charlen: usize,
    ) {
        #[rustfmt::skip]
        let strs = &[
            "",
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let s = strs[r.date().month() as usize];
        if !last_child_exists_and_has_expected_data(year, s) {
            year.children.push(base::tree::Node::new(s.into()))
        }
        let month = last_child(year);
        self.make_day_node(month, r, iid0, alignment_charlen);
    }

    fn make_year_node(
        &self,
        root: &mut base::tree::Node,
        r: &base::Record,
        iid0: usize,
        alignment_charlen: usize,
    ) {
        let buf = [
            (r.date().year() / 1000) as u8 + b'0',
            (r.date().year() / 100 % 10) as u8 + b'0',
            (r.date().year() / 10 % 10) as u8 + b'0',
            (r.date().year() % 10) as u8 + b'0',
        ];
        let s = std::str::from_utf8(&buf).expect("all chars should be ascii");
        if !last_child_exists_and_has_expected_data(root, s) {
            root.children
                .push(base::tree::Node::new(s.to_string().into()))
        }
        let year = last_child(root);
        self.make_month_node(year, r, iid0, alignment_charlen);
    }
}

fn last_child_exists_and_has_expected_data(node: &base::tree::Node, expected_data: &str) -> bool {
    match node.children.last() {
        Some(child) => child.data == expected_data,
        None => false,
    }
}

/// Panics if `node` does not have any children.
fn last_child(node: &mut base::tree::Node) -> &mut base::tree::Node {
    node.children
        .last_mut()
        .expect("a node should have just been added")
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use rstest::rstest;

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
    fn test_into_tree(#[case] first_iid: usize, #[case] rl: base::Recordlist, #[case] want: &str) {
        let config = Config {
            charset: base::Charset::default(),
            first_iid,
            leaf_string_postprocessor: None,
            rl,
        };
        let tr = config.to_tree();
        assert_eq!(tr.to_string(), want);
    }

    #[test]
    fn test_leaf_string_postprocessor() {
        let rl = r#"
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
        "#
        .parse::<base::Recordlist>()
        .unwrap();

        fn lspp(_: &Config, _: &base::Record, iid0: usize, mut leaf_string: String) -> String {
            match iid0 % 3 {
                0 => leaf_string.insert_str(0, "[0]"),
                1 => leaf_string.push_str("[1]"),
                _ => {}
            }
            leaf_string
        }

        let want = indoc!(
            "
            0000
            `-- Jan
                `-- 31st
                    `-- [0]1 ------------ 0.00  aaa
            2015
            `-- Mar
                |-- 30th
                |   |-- [0]1 ------------ 0.01  b
                |   |-- 2 ------------ 0.01  bb[1]
                |   |-- 3 ------------ 0.01  bbb
                |   |-- [0]4 ------------ 0.01  bbbb
                |   |-- 5 ------------ 0.01  bbbbb[1]
                |   |-- 6 ----------- (0.01) bbbbb
                |   |-- [0]7 ----------- (0.01) bbbb
                |   |-- 8 ----------- (0.01) bbb[1]
                |   |-- 9 ----------- (0.01) bb
                |   `-- [0]10 -- (1,234,567.89) b
                `-- 31st
                    `-- [0]1 -------- 1,234.56  ccc
            "
        );

        let config = Config {
            charset: base::Charset::default(),
            first_iid: 1,
            leaf_string_postprocessor: Some(Box::new(lspp)),
            rl,
        };
        let tr = config.to_tree();
        assert_eq!(tr.to_string(), want);
    }
}
