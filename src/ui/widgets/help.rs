//! Help view widget showing all keybindings.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

/// Help categories
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Navigation",
        &[
            ("↑/k", "Move up"),
            ("↓/j", "Move down"),
            ("←/h", "Move left / Previous"),
            ("→/l", "Move right / Next"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
            ("PgUp/b", "Page up"),
            ("PgDn/f", "Page down"),
        ],
    ),
    (
        "Selection & Actions",
        &[
            ("Enter/Space", "Select / Confirm"),
            ("Esc", "Back / Cancel"),
            ("q", "Quit"),
        ],
    ),
    (
        "Spec Operations",
        &[
            ("n", "Create new specification"),
            ("v", "View document (spec.md)"),
            ("e", "Edit document"),
            ("r", "Run workflow command"),
            ("F5", "Refresh specs & worktrees"),
        ],
    ),
    (
        "Worktree Management",
        &[
            ("w", "Switch to spec's worktree"),
            ("W", "Open worktree manager"),
            ("d", "Delete worktree (with confirm)"),
            ("D", "Force delete worktree"),
        ],
    ),
    (
        "Document View",
        &[
            ("1", "View spec.md"),
            ("2", "View plan.md"),
            ("3", "View tasks.md"),
            ("4", "View research.md"),
            ("e", "Switch to edit mode"),
        ],
    ),
    (
        "Document Edit",
        &[("Ctrl+S", "Save document"), ("Esc", "Close editor")],
    ),
    (
        "Command Output",
        &[
            ("c", "Cancel running command"),
            ("↑/↓", "Scroll output"),
            ("G", "Scroll to bottom"),
        ],
    ),
];

/// State for the help view
#[derive(Debug, Default, Clone)]
pub struct HelpViewState {
    /// Current scroll offset (in lines)
    pub scroll_offset: usize,
    /// Total number of lines
    pub total_lines: usize,
    /// Visible height
    pub visible_height: usize,
}

impl HelpViewState {
    /// Create a new help view state
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            total_lines: 0,
            visible_height: 0,
        }
    }

    /// Scroll up by n lines
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Scroll down by n lines
    pub fn scroll_down(&mut self, n: usize) {
        let max_offset = self.total_lines.saturating_sub(self.visible_height);
        self.scroll_offset = (self.scroll_offset + n).min(max_offset);
    }

    /// Page up
    pub fn page_up(&mut self) {
        self.scroll_up(self.visible_height.saturating_sub(2));
    }

    /// Page down
    pub fn page_down(&mut self) {
        self.scroll_down(self.visible_height.saturating_sub(2));
    }
}

/// Help view widget
pub struct HelpWidget<'a> {
    scroll_offset: usize,
    state: &'a mut HelpViewState,
}

impl<'a> HelpWidget<'a> {
    /// Create a new help widget
    pub fn new(state: &'a mut HelpViewState) -> Self {
        Self {
            scroll_offset: state.scroll_offset,
            state,
        }
    }

    /// Build help text lines
    fn build_lines() -> Vec<Line<'static>> {
        // Header
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "  speck Help  ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(Span::styled(
                "A TUI for spec-driven development with git worktree management.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];

        // Build sections
        for (section_name, bindings) in HELP_SECTIONS {
            // Section header
            lines.push(Line::from(Span::styled(
                format!("─── {} ───", section_name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            // Bindings
            for (key, description) in *bindings {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:12}", key), Style::default().fg(Color::Green)),
                    Span::raw(*description),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Footer
        lines.push(Line::from(Span::styled(
            "─────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Green)),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::styled(" to close help", Style::default().fg(Color::DarkGray)),
        ]));

        lines
    }
}

impl Widget for HelpWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        Clear.render(area, buf);

        // Build the help content
        let lines = Self::build_lines();

        // Update state
        self.state.total_lines = lines.len();
        self.state.visible_height = area.height.saturating_sub(2) as usize;

        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Help (?) ");

        let inner = block.inner(area);
        block.render(area, buf);

        // Skip lines based on scroll offset
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(self.scroll_offset)
            .take(inner.height as usize)
            .collect();

        // Render text
        let paragraph = Paragraph::new(visible_lines);
        paragraph.render(inner, buf);

        // Render scrollbar if content exceeds view
        if self.state.total_lines > self.state.visible_height {
            let mut scrollbar_state =
                ScrollbarState::new(self.state.total_lines).position(self.scroll_offset);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"));

            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_state_scroll() {
        let mut state = HelpViewState::new();
        state.total_lines = 50;
        state.visible_height = 20;

        state.scroll_down(5);
        assert_eq!(state.scroll_offset, 5);

        state.scroll_up(3);
        assert_eq!(state.scroll_offset, 2);

        state.scroll_up(10);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_help_lines_built() {
        let lines = HelpWidget::build_lines();
        assert!(!lines.is_empty());
        // Should have at least the header and some sections
        assert!(lines.len() > 10);
    }
}
