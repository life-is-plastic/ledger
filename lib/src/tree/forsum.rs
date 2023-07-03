use crate::tree::Node;
use crate::util;
use crate::Aggregate;
use crate::Cents;
use crate::Charset;
use crate::Recordlist;
use crate::Tree;

#[derive(Default)]
pub struct Config {
    pub charset: Charset,
    pub level: usize,
    pub rl: Recordlist,
}

impl Config {
    const IN: &str = "In";
    const OUT: &str = "Out";
    const NET: &str = "Net";
    const TOTAL: &str = "Total";

    pub fn to_tree<'a>(&'a self) -> Tree<'a> {
        let mut pos = Aggregate::<&str, Cents>::default();
        let mut neg = Aggregate::<&str, Cents>::default();
        for r in self.rl.iter() {
            match r.amount().cmp(&Cents(0)) {
                std::cmp::Ordering::Greater => pos.add(r.category().level(self.level), r.amount()),
                std::cmp::Ordering::Less => neg.add(r.category().level(self.level), r.amount()),
                _ => {}
            }
        }

        let sort_agg = |agg: &Aggregate<&'a str, Cents>| {
            let mut v = agg.iter().collect::<Vec<_>>();
            v.sort_unstable_by(|&(s1, a1), &(s2, a2)| {
                if a1 == a2 {
                    s1.cmp(s2)
                } else {
                    a2.0.abs().cmp(&a1.0.abs())
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

        fn char_count((label, amount): (&str, Cents)) -> usize {
            label.chars().count()
                + util::BOUNDING_SPACES_COUNT
                + util::MIN_DASHES_COUNT
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

        let mut root = Node::default();
        self.add_vec(&mut root, posv, Self::IN, alignment_charlen);
        self.add_vec(&mut root, negv, Self::OUT, alignment_charlen);
        self.add_vec(&mut root, totv, Self::NET, alignment_charlen);
        Tree {
            charset: &self.charset,
            root,
        }
    }

    fn leaf_data(&self, label: &str, amount: Cents, alignment_charlen: usize) -> String {
        let dash_count = alignment_charlen
            - label.chars().count()
            - util::BOUNDING_SPACES_COUNT
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

    fn add_vec(
        &self,
        root: &mut Node,
        v: Vec<(&str, Cents)>,
        name: &'static str,
        alignment_charlen: usize,
    ) {
        if v.is_empty() {
            return;
        }
        root.children.push(Node::new(name.into()));
        let node = root
            .children
            .last_mut()
            .expect("a node should have just been added");
        for (label, amount) in v {
            let data = self.leaf_data(label, amount, alignment_charlen);
            node.children.push(Node::new(data.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

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
    fn test_into_tree(#[case] level: usize, #[case] rl: Recordlist, #[case] want: &str) {
        let config = Config {
            charset: Charset::default(),
            level,
            rl,
        };
        let tr = config.to_tree();
        assert_eq!(tr.to_string(), want)
    }
}
