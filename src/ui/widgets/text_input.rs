//! Text input widget for single-line text entry.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Single-line text input widget
pub struct TextInputWidget<'a> {
    /// Current input value
    value: &'a str,
    /// Cursor position
    cursor: usize,
    /// Placeholder text when empty
    placeholder: &'a str,
    /// Title for the input box
    title: &'a str,
    /// Whether the input is focused
    focused: bool,
}

impl<'a> TextInputWidget<'a> {
    /// Create a new text input widget
    pub fn new(value: &'a str, cursor: usize) -> Self {
        Self {
            value,
            cursor,
            placeholder: "",
            title: "Input",
            focused: true,
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Set title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Set focused state
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Widget for TextInputWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(" {} ", self.title));

        let inner = block.inner(area);
        block.render(area, buf);

        // Display value or placeholder
        let display_text = if self.value.is_empty() {
            self.placeholder
        } else {
            self.value
        };

        let text_style = if self.value.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        // Render text with cursor
        if self.focused && !self.value.is_empty() {
            let before_cursor: String = self.value.chars().take(self.cursor).collect();
            let cursor_char: String = self.value.chars().skip(self.cursor).take(1).collect();
            let after_cursor: String = self.value.chars().skip(self.cursor + 1).collect();

            let mut x = inner.x;

            buf.set_string(x, inner.y, &before_cursor, text_style);
            x += before_cursor.len() as u16;

            // Render cursor
            let cursor_text = if cursor_char.is_empty() {
                " "
            } else {
                &cursor_char
            };
            buf.set_string(
                x,
                inner.y,
                cursor_text,
                Style::default().fg(Color::Black).bg(Color::White),
            );
            x += 1;

            buf.set_string(x, inner.y, &after_cursor, text_style);
        } else if self.focused && self.value.is_empty() {
            // Show cursor at start
            buf.set_string(
                inner.x,
                inner.y,
                " ",
                Style::default().fg(Color::Black).bg(Color::White),
            );
            if !self.placeholder.is_empty() {
                buf.set_string(
                    inner.x + 1,
                    inner.y,
                    self.placeholder,
                    Style::default().fg(Color::DarkGray),
                );
            }
        } else {
            buf.set_string(inner.x, inner.y, display_text, text_style);
        }
    }
}

/// State for text input
#[derive(Debug, Default, Clone)]
pub struct TextInputState {
    /// Current value
    pub value: String,
    /// Cursor position (character index)
    pub cursor: usize,
}

impl TextInputState {
    /// Create a new text input state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial value
    pub fn with_value(value: String) -> Self {
        let cursor = value.len();
        Self { value, cursor }
    }

    /// Handle a key event, returns true if the event was handled
    pub fn handle_key(&mut self, key: KeyEvent) -> TextInputAction {
        match key.code {
            KeyCode::Char(c) => {
                // Insert character at cursor
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return TextInputAction::None;
                }
                self.value.insert(self.cursor, c);
                self.cursor += 1;
                TextInputAction::Changed
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                    TextInputAction::Changed
                } else {
                    TextInputAction::None
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                    TextInputAction::Changed
                } else {
                    TextInputAction::None
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                TextInputAction::None
            }
            KeyCode::Right => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                }
                TextInputAction::None
            }
            KeyCode::Home => {
                self.cursor = 0;
                TextInputAction::None
            }
            KeyCode::End => {
                self.cursor = self.value.len();
                TextInputAction::None
            }
            KeyCode::Enter => TextInputAction::Submit,
            KeyCode::Esc => TextInputAction::Cancel,
            _ => TextInputAction::None,
        }
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

/// Actions that can result from text input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputAction {
    /// No action
    None,
    /// Value changed
    Changed,
    /// User submitted (Enter)
    Submit,
    /// User cancelled (Esc)
    Cancel,
}

/// Dialog for creating a new spec
pub struct NewSpecDialog<'a> {
    /// Input state
    input: &'a TextInputState,
    /// Error message to display
    error: Option<&'a str>,
}

impl<'a> NewSpecDialog<'a> {
    /// Create a new spec dialog
    pub fn new(input: &'a TextInputState) -> Self {
        Self { input, error: None }
    }

    /// Set error message
    pub fn error(mut self, error: Option<&'a str>) -> Self {
        self.error = error;
        self
    }
}

impl Widget for NewSpecDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        Clear.render(area, buf);

        // Render border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Create New Specification ");
        let inner = block.inner(area);
        block.render(area, buf);

        // Instructions
        let instructions =
            "Enter a short name for the new feature (e.g., 'user-auth', 'api-cache'):";
        buf.set_string(
            inner.x + 1,
            inner.y + 1,
            instructions,
            Style::default().fg(Color::White),
        );

        // Input field
        let input_area = Rect {
            x: inner.x + 1,
            y: inner.y + 3,
            width: inner.width.saturating_sub(2),
            height: 3,
        };

        let input_widget = TextInputWidget::new(&self.input.value, self.input.cursor)
            .title("Feature Name")
            .placeholder("feature-name");
        input_widget.render(input_area, buf);

        // Error message if any
        if let Some(error) = self.error {
            buf.set_string(
                inner.x + 1,
                inner.y + 7,
                error,
                Style::default().fg(Color::Red),
            );
        }

        // Help text
        let help_y = inner.y + inner.height.saturating_sub(2);
        buf.set_string(
            inner.x + 1,
            help_y,
            "Enter: Create | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_state() {
        let mut state = TextInputState::new();
        assert!(state.is_empty());

        // Type some text
        state.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        state.handle_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        assert_eq!(state.value(), "hi");
        assert_eq!(state.cursor, 2);

        // Backspace
        state.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(state.value(), "h");
        assert_eq!(state.cursor, 1);
    }

    #[test]
    fn test_text_input_navigation() {
        let mut state = TextInputState::with_value("hello".to_string());
        assert_eq!(state.cursor, 5);

        state.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(state.cursor, 0);

        state.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(state.cursor, 5);

        state.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(state.cursor, 4);
    }

    #[test]
    fn test_text_input_actions() {
        let mut state = TextInputState::new();

        let action = state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, TextInputAction::Submit);

        let action = state.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, TextInputAction::Cancel);
    }
}
