pub struct Charset {
    pub dash: char,
    pub tree_sideways_t: &'static str,
    pub tree_corner: &'static str,
    pub tree_pipe_gap: &'static str,
    pub tree_space: &'static str,
    pub chart_axis: char,
    pub chart_bar_pos: char,
    pub chart_bar_neg: char,
    pub color_prefix_green: &'static str,
    pub color_prefix_yellow: &'static str,
    pub color_prefix_red: &'static str,
    pub color_suffix: &'static str,
}

impl Default for Charset {
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
            color_prefix_green: "",
            color_prefix_yellow: "",
            color_prefix_red: "",
            color_suffix: "",
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
            color_prefix_green: "\x1b[38;2;90;165;90m",
            color_prefix_yellow: "\x1b[38;2;165;165;90m",
            color_prefix_red: "\x1b[38;2;165;90;90m",
            color_suffix: "\x1b[0m",
            ..self
        }
    }
}
