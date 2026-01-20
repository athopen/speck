//! Text editor widget using tui-textarea for document editing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

/// Editor widget for editing specification documents
pub struct EditorWidget<'a> {
    /// The text area for editing
    textarea: TextArea<'a>,
    /// Document title
    title: String,
    /// Whether the document has been modified
    modified: bool,
    /// Original content (for tracking modifications)
    original_content: String,
}

impl<'a> EditorWidget<'a> {
    /// Create a new editor widget
    pub fn new(content: &str, title: &str) -> Self {
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut textarea = TextArea::new(lines);

        // Configure the textarea
        textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray));
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));

        Self {
            textarea,
            title: title.to_string(),
            modified: false,
            original_content: content.to_string(),
        }
    }

    /// Check if the document has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get the current content
    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Mark the document as saved (reset modified flag)
    pub fn mark_saved(&mut self) {
        self.modified = false;
        self.original_content = self.content();
    }

    /// Handle a key event, returning true if the event was consumed
    pub fn handle_key(&mut self, key: KeyEvent) -> EditorAction {
        // Check for save shortcut (Ctrl+S)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            return EditorAction::Save;
        }

        // Check for quit shortcut (Ctrl+Q or Esc)
        if key.code == KeyCode::Esc {
            return EditorAction::Quit;
        }

        // Convert crossterm key event to tui-textarea input
        let input = convert_key_event(key);
        self.textarea.input(input);

        // Check if content changed
        let current = self.content();
        if current != self.original_content {
            self.modified = true;
        }

        EditorAction::None
    }

    /// Get the title with modification indicator
    pub fn display_title(&self) -> String {
        if self.modified {
            format!(" {} [Modified] ", self.title)
        } else {
            format!(" {} ", self.title)
        }
    }

    /// Get cursor position as (line, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    /// Get total line count
    pub fn line_count(&self) -> usize {
        self.textarea.lines().len()
    }
}

impl Widget for &EditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // We need to render through the textarea's widget
        // First create a block
        let border_style = if self.modified {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(self.display_title());

        // Set the block on a clone of textarea for rendering
        // Note: TextArea doesn't implement Clone, so we render manually
        block.render(area, buf);

        // Render inner content area
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Create a simple text display for now
        // (tui-textarea has its own widget rendering)
        let lines: Vec<Line> = self
            .textarea
            .lines()
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let line_num = format!("{:4} ", idx + 1);
                let (cursor_line, cursor_col) = self.cursor_position();

                let line_style = if idx == cursor_line {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                // Highlight cursor position
                if idx == cursor_line {
                    let mut spans = vec![Span::styled(line_num, Style::default().fg(Color::DarkGray))];

                    if cursor_col < line.len() {
                        let before: String = line.chars().take(cursor_col).collect();
                        let cursor_char: String = line.chars().skip(cursor_col).take(1).collect();
                        let after: String = line.chars().skip(cursor_col + 1).collect();

                        spans.push(Span::styled(before, line_style));
                        spans.push(Span::styled(
                            if cursor_char.is_empty() { " ".to_string() } else { cursor_char },
                            Style::default().bg(Color::White).fg(Color::Black),
                        ));
                        spans.push(Span::styled(after, line_style));
                    } else {
                        spans.push(Span::styled(line.to_string(), line_style));
                        spans.push(Span::styled(" ", Style::default().bg(Color::White).fg(Color::Black)));
                    }

                    Line::from(spans)
                } else {
                    Line::from(vec![
                        Span::styled(line_num, Style::default().fg(Color::DarkGray)),
                        Span::styled(line.to_string(), line_style),
                    ])
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);
    }
}

/// Actions that can result from editor key handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorAction {
    /// No action needed
    None,
    /// Save the document
    Save,
    /// Quit the editor
    Quit,
}

/// Convert crossterm KeyEvent to tui-textarea Input
fn convert_key_event(key: KeyEvent) -> Input {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    let key = match key.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Tab => Key::Tab,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Esc => Key::Esc,
        KeyCode::F(n) => Key::F(n),
        _ => Key::Null,
    };

    Input { key, ctrl, alt, shift }
}

/// Editor state that persists across renders
pub struct EditorState {
    /// The editor widget
    editor: Option<EditorWidget<'static>>,
    /// File path being edited
    file_path: Option<std::path::PathBuf>,
}

impl EditorState {
    /// Create a new editor state
    pub fn new() -> Self {
        Self {
            editor: None,
            file_path: None,
        }
    }

    /// Open a file for editing
    pub fn open(&mut self, content: String, title: String, path: std::path::PathBuf) {
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut textarea = TextArea::new(lines);
        textarea.set_cursor_line_style(Style::default().bg(Color::DarkGray));

        self.editor = Some(EditorWidget {
            textarea,
            title,
            modified: false,
            original_content: content,
        });
        self.file_path = Some(path);
    }

    /// Check if editor is active
    pub fn is_active(&self) -> bool {
        self.editor.is_some()
    }

    /// Get the editor widget
    pub fn editor(&self) -> Option<&EditorWidget<'static>> {
        self.editor.as_ref()
    }

    /// Get mutable editor widget
    pub fn editor_mut(&mut self) -> Option<&mut EditorWidget<'static>> {
        self.editor.as_mut()
    }

    /// Get file path
    pub fn file_path(&self) -> Option<&std::path::PathBuf> {
        self.file_path.as_ref()
    }

    /// Close the editor
    pub fn close(&mut self) {
        self.editor = None;
        self.file_path = None;
    }

    /// Check if modified
    pub fn is_modified(&self) -> bool {
        self.editor.as_ref().map_or(false, |e| e.is_modified())
    }

    /// Get content
    pub fn content(&self) -> Option<String> {
        self.editor.as_ref().map(|e| e.content())
    }

    /// Mark as saved
    pub fn mark_saved(&mut self) {
        if let Some(ref mut editor) = self.editor {
            editor.mark_saved();
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = EditorWidget::new("Hello\nWorld", "test.md");
        assert!(!editor.is_modified());
        assert_eq!(editor.line_count(), 2);
    }

    #[test]
    fn test_editor_content() {
        let editor = EditorWidget::new("Line 1\nLine 2\nLine 3", "test.md");
        assert_eq!(editor.content(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_editor_state() {
        let mut state = EditorState::new();
        assert!(!state.is_active());

        state.open(
            "Test content".to_string(),
            "test.md".to_string(),
            std::path::PathBuf::from("/tmp/test.md"),
        );
        assert!(state.is_active());
        assert!(!state.is_modified());

        state.close();
        assert!(!state.is_active());
    }
}
