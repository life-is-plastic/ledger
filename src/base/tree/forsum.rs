use crate::base;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub charset: base::Charset,
    pub level: usize,
    pub rl: base::Recordlist,
}

impl Config {
    const IN: &str = "In";
    const OUT: &str = "Out";
    const NET: &str = "Net";
    const TOTAL: &str = "Total";

    pub fn to_tree<'a>(&'a self) -> base::Tree<'a> {
        let mut pos = base::Aggregate::<&str, base::Cents>::default();
        let mut neg = base::Aggregate::<&str, base::Cents>::default();
        for r in self.rl.iter() {
            match r.amount().cmp(&base::Cents(0)) {
                std::cmp::Ordering::Greater => pos.add(r.category().level(self.level), r.amount()),
                std::cmp::Ordering::Less => neg.add(r.category().level(self.level), r.amount()),
                _ => {}
            }
        }

        let sort_agg = |agg: &base::Aggregate<&'a str, base::Cents>| {
            let mut v = agg.iter().collect::<Vec<_>>();
            v.sort_unstable_by(|&(s1, a1), &(s2, a2)| {
                if a1 == a2 {
                    s1.cmp(s2)
                } else {
                    a2.abs().cmp(&a1.abs())
                }
            });
            v
        };
        let posv = sort_agg(&pos);
        let negv = sort_agg(&neg);
        let totv = vec![
            (Self::IN, pos.sum()),
            (Self::OUT, neg.sum()),
            (Self::TOTAL, pos.sum() + neg.sum()),
        ];

        fn char_count((label, amount): (&str, base::Cents)) -> usize {
            label.chars().count()
                + base::util::BOUNDING_SPACES_COUNT
                + base::util::MIN_DASHES_COUNT
                + amount.charlen_for_alignment()
        }
        let alignment_charlen = [posv.as_slice(), negv.as_slice(), totv.as_slice()]
            .iter()
            .copied()
            .flatten()
            .copied()
            .map(char_count)
            .max()
            .unwrap_or_default();

        let mut root = base::tree::Node::default();
        self.add_vec_to_tree(&mut root, posv, Self::IN, alignment_charlen);
        self.add_vec_to_tree(&mut root, negv, Self::OUT, alignment_charlen);
        self.add_vec_to_tree(&mut root, totv, Self::NET, alignment_charlen);
        base::Tree {
            charset: &self.charset,
            root,
        }
    }

    fn leaf_data(&self, label: &str, amount: base::Cents, alignment_charlen: usize) -> String {
        let dash_count = alignment_charlen
            - label.chars().count()
            - base::util::BOUNDING_SPACES_COUNT
            - amount.charlen_for_alignment();
        let mut s = String::with_capacity(alignment_charlen);
        s.push_str(label);
        s.push(' ');
        for _ in 0..dash_count {
            s.push(self.charset.dash)
        }
        s.push(' ');
        s.push_str(&amount.to_string());
        s
    }

    fn add_vec_to_tree(
        &self,
        root: &mut base::tree::Node,
        v: Vec<(&str, base::Cents)>,
        name: &'static str,
        alignment_charlen: usize,
    ) {
        if v.is_empty() {
            return;
        }
        root.children.push(base::tree::Node::new(name.into()));
        let node = root
            .children
            .last_mut()
            .expect("a node should have just been added");
        for (label, amount) in v {
            let data = self.leaf_data(label, amount, alignment_charlen);
            node.children.push(base::tree::Node::new(data.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use rstest::rstest;

    #[rstest]
    #[case(
        0,
        "",
        indoc!("
            Net
            |-- In ----- 0.00
            |-- Out ---- 0.00
            `-- Total -- 0.00
        "),
    )]
    #[case(
        0,
        r#"
            {"d":"2015-03-30","c":"a/b/c","a":111}
            {"d":"2015-03-30","c":"a/b/c","a":111}
        "#,
        indoc!("
            In
            `-- All ---- 2.22
            Net
            |-- In ----- 2.22
            |-- Out ---- 0.00
            `-- Total -- 2.22
        "),
    )]
    #[case(
        1,
        r#"
            {"d":"2015-03-30","c":"a/b/c","a":111}
            {"d":"2015-03-30","c":"a/b/c","a":111}
        "#,
        indoc!("
            In
            `-- a ------ 2.22
            Net
            |-- In ----- 2.22
            |-- Out ---- 0.00
            `-- Total -- 2.22
        "),
    )]
    #[case(
        3,
        r#"
            {"d":"2015-03-30","c":"a/b/c","a":111}
            {"d":"2015-03-30","c":"a/b/c","a":111}
        "#,
        indoc!("
            In
            `-- a/b/c -- 2.22
            Net
            |-- In ----- 2.22
            |-- Out ---- 0.00
            `-- Total -- 2.22
        "),
    )]
    #[case(
        100,
        r#"
            {"d":"2015-03-30","c":"a/b/c","a":111}
            {"d":"2015-03-30","c":"a/b/c/d/e","a":-12345}
        "#,
        indoc!("
            In
            `-- a/b/c --------- 1.11
            Out
            `-- a/b/c/d/e -- (123.45)
            Net
            |-- In ------------ 1.11
            |-- Out -------- (123.45)
            `-- Total ------ (122.34)
        "),
    )]
    #[case(
        100,
        r#"
            {"d":"2015-03-29","c":"bbb","a":-111}
            {"d":"2015-03-30","c":"aaa","a":-111}
            {"d":"2015-03-30","c":"ccc","a":-12345}
        "#,
        indoc!("
            Out
            |-- ccc ---- (123.45)
            |-- aaa ------ (1.11)
            `-- bbb ------ (1.11)
            Net
            |-- In -------- 0.00
            |-- Out ---- (125.67)
            `-- Total -- (125.67)
        "),
    )]
    fn test_into_tree(#[case] level: usize, #[case] rl: base::Recordlist, #[case] want: &str) {
        let config = Config {
            charset: base::Charset::default(),
            level,
            rl,
        };
        let tr = config.to_tree();
        assert_eq!(tr.to_string(), want)
    }
}
