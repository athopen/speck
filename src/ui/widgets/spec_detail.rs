//! Document viewer widget with markdown syntax highlighting.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

/// Widget for viewing specification documents with markdown highlighting
pub struct SpecDetailWidget<'a> {
    /// Document content
    content: &'a str,
    /// Document title
    title: String,
    /// Scroll offset
    scroll_offset: usize,
    /// Total lines in document
    total_lines: usize,
    /// Visible height (for scrollbar)
    visible_height: usize,
}

impl<'a> SpecDetailWidget<'a> {
    /// Create a new spec detail widget
    pub fn new(content: &'a str, title: &str) -> Self {
        let total_lines = content.lines().count();
        Self {
            content,
            title: title.to_string(),
            scroll_offset: 0,
            total_lines,
            visible_height: 20,
        }
    }

    /// Set the scroll offset
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set the visible height (for scrollbar calculations)
    pub fn visible_height(mut self, height: usize) -> Self {
        self.visible_height = height;
        self
    }

    /// Apply markdown-aware syntax highlighting to the content
    fn highlight_content(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        for line in self.content.lines() {
            let styled_line = self.highlight_line(line);
            lines.push(styled_line);
        }

        lines
    }

    /// Highlight a single line based on markdown patterns
    fn highlight_line(&self, line: &str) -> Line<'a> {
        let trimmed = line.trim_start();

        // Headers
        if trimmed.starts_with("# ") {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        if trimmed.starts_with("## ") {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        if trimmed.starts_with("### ") {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        if trimmed.starts_with("#### ")
            || trimmed.starts_with("##### ")
            || trimmed.starts_with("###### ")
        {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Code blocks (```  or indented)
        if trimmed.starts_with("```") {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // Horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }

        // List items
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let indent_spaces = line.len() - trimmed.len();
            let indent = " ".repeat(indent_spaces);
            return Line::from(vec![
                Span::raw(indent),
                Span::styled(
                    trimmed.chars().take(2).collect::<String>(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(trimmed.chars().skip(2).collect::<String>()),
            ]);
        }

        // Numbered list items
        if let Some(pos) = trimmed.find(". ") {
            if pos <= 3 && trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                let indent_spaces = line.len() - trimmed.len();
                let indent = " ".repeat(indent_spaces);
                let number_part: String = trimmed.chars().take(pos + 2).collect();
                let rest: String = trimmed.chars().skip(pos + 2).collect();
                return Line::from(vec![
                    Span::raw(indent),
                    Span::styled(number_part, Style::default().fg(Color::Yellow)),
                    Span::raw(rest),
                ]);
            }
        }

        // Checkbox items
        if trimmed.starts_with("- [ ] ")
            || trimmed.starts_with("- [x] ")
            || trimmed.starts_with("- [X] ")
        {
            let indent_spaces = line.len() - trimmed.len();
            let indent = " ".repeat(indent_spaces);
            let checkbox: String = trimmed.chars().take(6).collect();
            let rest: String = trimmed.chars().skip(6).collect();
            let checkbox_style = if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            return Line::from(vec![
                Span::raw(indent),
                Span::styled(checkbox, checkbox_style),
                Span::raw(rest),
            ]);
        }

        // Blockquotes
        if trimmed.starts_with("> ") {
            return Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ));
        }

        // Bold text **text** or __text__ - simple inline highlighting
        if line.contains("**") || line.contains("__") {
            return self.highlight_inline_formatting(line);
        }

        // Regular text
        Line::from(line.to_string())
    }

    /// Highlight inline formatting like **bold** and *italic*
    fn highlight_inline_formatting(&self, line: &str) -> Line<'a> {
        let mut spans = Vec::new();
        let mut chars = line.chars().peekable();
        let mut current = String::new();
        let mut in_bold = false;
        let mut in_italic = false;
        let mut in_code = false;

        while let Some(c) = chars.next() {
            if c == '`' && !in_bold && !in_italic {
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }
                if in_code {
                    spans.push(Span::styled(
                        current.clone(),
                        Style::default().fg(Color::Green).bg(Color::DarkGray),
                    ));
                    current.clear();
                }
                in_code = !in_code;
                continue;
            }

            if in_code {
                current.push(c);
                continue;
            }

            if c == '*' && chars.peek() == Some(&'*') {
                chars.next(); // consume second *
                if !current.is_empty() {
                    let style = if in_bold {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(current.clone(), style));
                    current.clear();
                }
                in_bold = !in_bold;
                continue;
            }

            if c == '*' && !in_bold {
                if !current.is_empty() {
                    let style = if in_italic {
                        Style::default().add_modifier(Modifier::ITALIC)
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(current.clone(), style));
                    current.clear();
                }
                in_italic = !in_italic;
                continue;
            }

            current.push(c);
        }

        if !current.is_empty() {
            let mut style = Style::default();
            if in_bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if in_italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            spans.push(Span::styled(current, style));
        }

        if spans.is_empty() {
            Line::from(line.to_string())
        } else {
            Line::from(spans)
        }
    }
}

impl Widget for SpecDetailWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.highlight_content();
        let total_lines = lines.len();

        // Create the block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(" {} ", self.title));

        let inner = block.inner(area);

        // Render the block
        block.render(area, buf);

        // Calculate visible range
        let visible_height = inner.height as usize;
        let scroll = self
            .scroll_offset
            .min(total_lines.saturating_sub(visible_height));

        // Create paragraph with scroll
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(scroll)
            .take(visible_height)
            .collect();

        let paragraph = Paragraph::new(visible_lines);
        paragraph.render(inner, buf);

        // Render scrollbar if needed
        if total_lines > visible_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(scroll)
                .viewport_content_length(visible_height);

            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);
        }
    }
}

/// Document viewer state for scrolling
#[derive(Debug, Default)]
pub struct DocumentViewerState {
    /// Current scroll offset
    scroll_offset: usize,
    /// Total lines in document
    total_lines: usize,
    /// Visible height
    visible_height: usize,
}

impl DocumentViewerState {
    /// Create a new document viewer state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the total lines
    pub fn set_total_lines(&mut self, lines: usize) {
        self.total_lines = lines;
    }

    /// Set the visible height
    pub fn set_visible_height(&mut self, height: usize) {
        self.visible_height = height;
    }

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scroll up by amount
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll down by amount
    pub fn scroll_down(&mut self, amount: usize) {
        let max_scroll = self.total_lines.saturating_sub(self.visible_height);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }

    /// Jump to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Jump to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_lines.saturating_sub(self.visible_height);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_viewer_state_scroll() {
        let mut state = DocumentViewerState::new();
        state.set_total_lines(100);
        state.set_visible_height(20);

        assert_eq!(state.scroll_offset(), 0);

        state.scroll_down(10);
        assert_eq!(state.scroll_offset(), 10);

        state.scroll_up(5);
        assert_eq!(state.scroll_offset(), 5);

        state.scroll_to_bottom();
        assert_eq!(state.scroll_offset(), 80);

        state.scroll_to_top();
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_widget_creation() {
        let content = "# Test\n\nSome content";
        let widget = SpecDetailWidget::new(content, "test.md");
        assert_eq!(widget.total_lines, 3);
    }
}
