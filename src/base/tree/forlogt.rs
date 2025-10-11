use crate::base;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub charset: base::Charset,
    pub templates: std::collections::BTreeMap<String, Vec<base::config::TemplateEntry>>,
}

impl Config {
    pub fn to_tree(&self) -> base::Tree {
        fn char_count(entry: &base::config::TemplateEntry) -> usize {
            entry.category.as_str().chars().count()
                + base::util::BOUNDING_SPACES_COUNT
                + base::util::MIN_DASHES_COUNT
                + entry.amount.charlen_for_alignment()
        }
        let alignment_charlen = self
            .templates
            .values()
            .flatten()
            .map(char_count)
            .max()
            .unwrap_or_default();

        let mut root = base::tree::Node::default();
        for (tmpl_name, entries) in self.templates.iter() {
            let mut name_node = base::tree::Node::new(tmpl_name.clone().into());
            for entry in entries.iter() {
                name_node.children.push(base::tree::Node::new(
                    self.leaf_data(entry, alignment_charlen).into(),
                ));
            }
            root.children.push(name_node)
        }
        base::Tree {
            charset: self.charset.clone(),
            root,
        }
    }

    fn leaf_data(&self, entry: &base::config::TemplateEntry, alignment_charlen: usize) -> String {
        let dash_count = alignment_charlen
            - entry.category.as_str().chars().count()
            - base::util::BOUNDING_SPACES_COUNT
            - entry.amount.charlen_for_alignment();
        let mut s = String::with_capacity(alignment_charlen);
        s.push_str(entry.category.as_str());
        s.push(' ');
        for _ in 0..dash_count {
            s.push(self.charset.dash)
        }
        s.push(' ');
        s.push_str(&entry.amount.to_string());
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use rstest::rstest;

    #[rstest]
    #[case(Default::default(), "")]
    #[case(
        [
            (
                String::from("template1"),
                vec![
                    base::config::TemplateEntry {
                        category: base::Category::try_from("category1").unwrap(),
                        amount: base::Cents(123),
                    },
                    base::config::TemplateEntry {
                        category: base::Category::try_from("Category2").unwrap(),
                        amount: base::Cents(-123),
                    },
                ],
            ),
            (
                String::from("Template2"),
                vec![],
            ),
        ].into(),
        indoc!("
            Template2
            template1
            |-- category1 --- 1.23
            `-- Category2 -- (1.23)
        "),
    )]
    fn test_to_tree(
        #[case] templates: std::collections::BTreeMap<String, Vec<base::config::TemplateEntry>>,
        #[case] want: &str,
    ) {
        let config = Config {
            charset: base::Charset::default(),
            templates,
        };
        let tr = config.to_tree();
        assert_eq!(tr.to_string(), want)
    }
}
