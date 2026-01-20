//! Output panel widget for displaying streaming command output.

use crate::domain::{ExecutionState, OutputLine, OutputStream, WorkflowCommand};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};
use std::fmt;
use std::time::Instant;

/// Maximum number of lines to keep in the output buffer
const MAX_OUTPUT_LINES: usize = 1000;

/// Widget for displaying workflow command output
pub struct OutputPanelWidget<'a> {
    /// Output lines to display
    lines: &'a [OutputLine],
    /// Command being executed (if any)
    command: Option<&'a WorkflowCommand>,
    /// Scroll offset
    scroll_offset: usize,
    /// Is auto-scroll enabled
    auto_scroll: bool,
}

impl<'a> OutputPanelWidget<'a> {
    /// Create a new output panel widget
    pub fn new(lines: &'a [OutputLine], command: Option<&'a WorkflowCommand>) -> Self {
        Self {
            lines,
            command,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    /// Set the scroll offset
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self.auto_scroll = false;
        self
    }

    /// Enable auto-scroll to bottom
    pub fn auto_scroll(mut self, enabled: bool) -> Self {
        self.auto_scroll = enabled;
        self
    }

    /// Get the title based on command state
    fn title(&self) -> String {
        match self.command {
            Some(cmd) => {
                let state_indicator = match &cmd.state {
                    ExecutionState::Pending => "⏳",
                    ExecutionState::Running { .. } => "▶",
                    ExecutionState::Completed { exit_code, .. } if *exit_code == 0 => "✓",
                    ExecutionState::Completed { .. } => "✗",
                    ExecutionState::Failed { .. } => "✗",
                    ExecutionState::Cancelled => "⊘",
                };
                let state_name = match &cmd.state {
                    ExecutionState::Pending => "Pending",
                    ExecutionState::Running { .. } => "Running",
                    ExecutionState::Completed { .. } => "Completed",
                    ExecutionState::Failed { .. } => "Failed",
                    ExecutionState::Cancelled => "Cancelled",
                };
                format!(
                    " {} {} - {} ",
                    state_indicator,
                    cmd.command_type.display_name(),
                    state_name
                )
            }
            None => " Output ".to_string(),
        }
    }

    /// Get the border style based on command state
    fn border_style(&self) -> Style {
        match self.command.map(|c| &c.state) {
            Some(ExecutionState::Running { .. }) => Style::default().fg(Color::Yellow),
            Some(ExecutionState::Completed { exit_code, .. }) if *exit_code == 0 => {
                Style::default().fg(Color::Green)
            }
            Some(ExecutionState::Completed { .. }) | Some(ExecutionState::Failed { .. }) => {
                Style::default().fg(Color::Red)
            }
            Some(ExecutionState::Cancelled) => Style::default().fg(Color::DarkGray),
            _ => Style::default(),
        }
    }

    /// Format output lines for display
    fn format_lines(&self) -> Vec<Line<'a>> {
        self.lines
            .iter()
            .map(|line| {
                let style = if line.stream == OutputStream::Stderr {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };

                // Add timestamp prefix
                let prefix = format_timestamp(&line.timestamp);

                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", prefix),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(line.content.clone(), style),
                ])
            })
            .collect()
    }
}

impl Widget for OutputPanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = self.format_lines();
        let total_lines = lines.len();

        // Calculate scroll position
        let scroll = if self.auto_scroll && total_lines > area.height as usize {
            total_lines.saturating_sub(area.height.saturating_sub(2) as usize)
        } else {
            self.scroll_offset
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.border_style())
                    .title(self.title()),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0));

        paragraph.render(area, buf);

        // Render scrollbar if there are more lines than visible
        if total_lines > area.height.saturating_sub(2) as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(total_lines)
                .position(scroll)
                .viewport_content_length(area.height.saturating_sub(2) as usize);

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

/// Format a timestamp for display
fn format_timestamp(ts: &Instant) -> String {
    // We can't easily get wall clock from Instant, so just show elapsed since some reference
    // This is a simplification - in practice you'd track start time separately
    let elapsed = ts.elapsed();
    let secs = elapsed.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Output buffer for collecting command output
#[derive(Debug, Default)]
pub struct OutputBuffer {
    /// Output lines
    lines: Vec<OutputLine>,
    /// Start time
    start_time: Option<Instant>,
    /// Scroll offset (for manual scrolling)
    scroll_offset: usize,
    /// Is auto-scroll enabled
    auto_scroll: bool,
}

impl OutputBuffer {
    /// Create a new output buffer
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            start_time: None,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    /// Start a new command (clear buffer and reset timer)
    pub fn start(&mut self) {
        self.lines.clear();
        self.start_time = Some(Instant::now());
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    /// Add a line to the buffer
    pub fn push(&mut self, content: String, stream: OutputStream) {
        let line = OutputLine {
            content,
            stream,
            timestamp: Instant::now(),
        };
        self.lines.push(line);

        // Trim if too many lines
        if self.lines.len() > MAX_OUTPUT_LINES {
            self.lines.remove(0);
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }

    /// Add stdout line
    pub fn push_stdout(&mut self, content: String) {
        self.push(content, OutputStream::Stdout);
    }

    /// Add stderr line
    pub fn push_stderr(&mut self, content: String) {
        self.push(content, OutputStream::Stderr);
    }

    /// Get all lines
    pub fn lines(&self) -> &[OutputLine] {
        &self.lines
    }

    /// Get line count
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.lines.clear();
        self.start_time = None;
        self.scroll_offset = 0;
    }

    /// Scroll up
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.auto_scroll = false;
    }

    /// Scroll down
    pub fn scroll_down(&mut self, amount: usize, visible_height: usize) {
        let max_scroll = self.lines.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);

        // Re-enable auto-scroll if at bottom
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    /// Jump to bottom and enable auto-scroll
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Check if auto-scroll is enabled
    pub fn is_auto_scroll(&self) -> bool {
        self.auto_scroll
    }
}

impl fmt::Display for OutputBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content: Vec<_> = self.lines.iter().map(|l| l.content.as_str()).collect();
        write!(f, "{}", content.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_buffer_push() {
        let mut buffer = OutputBuffer::new();
        buffer.start();

        buffer.push_stdout("Hello".to_string());
        buffer.push_stderr("Error".to_string());

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.lines()[0].stream, OutputStream::Stdout);
        assert_eq!(buffer.lines()[1].stream, OutputStream::Stderr);
    }

    #[test]
    fn test_output_buffer_scroll() {
        let mut buffer = OutputBuffer::new();
        buffer.start();

        for i in 0..100 {
            buffer.push_stdout(format!("Line {}", i));
        }

        assert!(buffer.is_auto_scroll());

        buffer.scroll_up(10);
        assert!(!buffer.is_auto_scroll());

        buffer.scroll_to_bottom();
        assert!(buffer.is_auto_scroll());
    }

    #[test]
    fn test_output_buffer_max_lines() {
        let mut buffer = OutputBuffer::new();
        buffer.start();

        for i in 0..MAX_OUTPUT_LINES + 100 {
            buffer.push_stdout(format!("Line {}", i));
        }

        assert_eq!(buffer.len(), MAX_OUTPUT_LINES);
    }
}
