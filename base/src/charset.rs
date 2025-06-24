#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charset {
    pub dash: char,
    pub tree_sideways_t: &'static str,
    pub tree_corner: &'static str,
    pub tree_pipe_gap: &'static str,
    pub tree_space: &'static str,
    pub chart_axis: char,
    pub chart_bar_pos: char,
    pub chart_bar_neg: char,
    pub color: bool,
}

impl Default for Charset {
    /// Only ASCII characters. No color.
    fn default() -> Self {
        Self {
            dash: '-',
            tree_sideways_t: "|-- ",
            tree_corner: "`-- ",
            tree_pipe_gap: "|   ",
            tree_space: "    ",
            chart_axis: '|',
            chart_bar_pos: '+',
            chart_bar_neg: '-',
            color: false,
        }
    }
}

impl Charset {
    pub fn with_unicode(self) -> Self {
        Self {
            dash: '\u{2500}',
            tree_sideways_t: "\u{251c}\u{2500}\u{2500} ",
            tree_corner: "\u{2514}\u{2500}\u{2500} ",
            tree_pipe_gap: "\u{2502}   ",
            tree_space: "    ",
            chart_axis: '\u{2502}',
            chart_bar_pos: '\u{2588}',
            chart_bar_neg: '\u{2588}',
            ..self
        }
    }

    pub fn with_color(self) -> Self {
        Self {
            color: true,
            ..self
        }
    }
}
