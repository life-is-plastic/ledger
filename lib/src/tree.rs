pub mod forsum;
pub mod forview;

use crate::Charset;

pub struct Tree<'cs> {
    charset: &'cs Charset,
    root: Node,
}

struct Node {
    data: std::borrow::Cow<'static, str>,
    children: Vec<Self>,
}

impl Node {
    fn new(data: std::borrow::Cow<'static, str>) -> Self {
        Self {
            data,
            children: Vec::new(),
        }
    }
}

impl Default for Node {
    /// Returns a node suitable for acting as the root node.
    fn default() -> Self {
        Self {
            data: "".into(),
            children: Default::default(),
        }
    }
}

impl Tree<'_> {
    pub fn is_empty(&self) -> bool {
        self.root.children.is_empty()
    }
}

impl std::fmt::Display for Tree<'_> {
    /// Writes a terminating newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_node(
            node: &Node,
            f: &mut std::fmt::Formatter,
            preprefix: &mut String,
            is_last_child_of_parent: bool,
            charset: &Charset,
        ) -> std::fmt::Result {
            let (prefix_tail, child_prefix_tail) = if is_last_child_of_parent {
                (charset.tree_corner, charset.tree_space)
            } else {
                (charset.tree_sideways_t, charset.tree_pipe_gap)
            };
            writeln!(f, "{}{}{}", preprefix, prefix_tail, node.data)?;
            preprefix.push_str(child_prefix_tail);
            for (i, child) in node.children.iter().enumerate() {
                write_node(child, f, preprefix, i >= node.children.len() - 1, charset)?;
            }
            preprefix.truncate(preprefix.len() - child_prefix_tail.len());
            Ok(())
        }

        let mut preprefix = String::new();
        for lv1 in self.root.children.iter() {
            writeln!(f, "{}", lv1.data)?;
            for (i, lv2) in lv1.children.iter().enumerate() {
                write_node(
                    lv2,
                    f,
                    &mut preprefix,
                    i >= lv1.children.len() - 1,
                    self.charset,
                )?;
            }
        }
        Ok(())
    }
}

impl<'cs> From<forsum::Config<'cs, '_>> for Tree<'cs> {
    fn from(config: forsum::Config<'cs, '_>) -> Self {
        config.into_tree()
    }
}

impl<'cs> From<forview::Config<'cs, '_>> for Tree<'cs> {
    fn from(config: forview::Config<'cs, '_>) -> Self {
        config.into_tree()
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_to_string() {
        let mut tr = Tree {
            root: Node::default(),
            charset: &Charset::default(),
        };
        assert!(tr.is_empty());
        assert_eq!(tr.to_string(), "");

        macro_rules! push {
            ($node:ident, $data:expr) => {{
                $node.children.push(Node::new($data.into()));
                $node.children.last_mut().unwrap()
            }};
        }

        let root = &mut tr.root;
        let a = push!(root, "a");
        {
            let a1 = push!(a, "a1".to_string());
            {
                push!(a1, "a1a");
                push!(a1, "a1b");
            }
            let a2 = push!(a, "a2");
            {
                push!(a2, "a2a".to_string());
                push!(a2, "a2b");
                push!(a2, "a2c");
            }
        }
        push!(root, "b");
        let c = push!(root, "c");
        {
            push!(c, "c1");
            push!(c, "c2");
            let c3 = push!(c, "c3");
            {
                push!(c3, "c3a");
                push!(c3, "c3b");
                push!(c3, "c3c");
            }
        }

        assert!(!tr.is_empty());
        assert_eq!(
            tr.to_string(),
            indoc!(
                "
                a
                |-- a1
                |   |-- a1a
                |   `-- a1b
                `-- a2
                    |-- a2a
                    |-- a2b
                    `-- a2c
                b
                c
                |-- c1
                |-- c2
                `-- c3
                    |-- c3a
                    |-- c3b
                    `-- c3c
                "
            )
        )
    }
}
