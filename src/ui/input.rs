//! Keyboard input handling with vim-style navigation support.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Standard navigation mode
    #[default]
    Normal,
    /// Text editing mode
    Insert,
    /// Command palette / search mode
    Command,
}

/// Actions that can be triggered by keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    Home,
    End,

    // Selection
    Select,
    Back,

    // Workflow
    SwitchWorktree,
    ManageWorktrees,
    RunWorkflow,
    ViewDocument,
    EditDocument,
    NewSpec,
    DeleteWorktree,
    CancelCommand,

    // Misc
    Help,
    Quit,
    Refresh,
}

/// Keyboard bindings configuration
pub struct KeyBindings {
    pub vim_navigation: bool,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            vim_navigation: true,
        }
    }
}

/// Input handler for processing keyboard events
pub struct InputHandler {
    bindings: KeyBindings,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new(vim_navigation: bool) -> Self {
        Self {
            bindings: KeyBindings { vim_navigation },
        }
    }

    /// Handle a key event and return the corresponding action
    pub fn handle_key(&self, key: KeyEvent, mode: InputMode) -> Option<Action> {
        match mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Insert => self.handle_insert_key(key),
            InputMode::Command => self.handle_command_key(key),
        }
    }

    /// Handle key in normal mode
    fn handle_normal_key(&self, key: KeyEvent) -> Option<Action> {
        // Check for Ctrl+C first
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Some(Action::CancelCommand);
        }

        match key.code {
            // Navigation - arrow keys always work
            KeyCode::Up => Some(Action::MoveUp),
            KeyCode::Down => Some(Action::MoveDown),
            KeyCode::Left => Some(Action::MoveLeft),
            KeyCode::Right => Some(Action::MoveRight),
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Home => Some(Action::Home),
            KeyCode::End => Some(Action::End),

            // Vim-style navigation (j/k/h/l)
            KeyCode::Char('j') if self.bindings.vim_navigation => Some(Action::MoveDown),
            KeyCode::Char('k') if self.bindings.vim_navigation => Some(Action::MoveUp),
            KeyCode::Char('h') if self.bindings.vim_navigation => Some(Action::MoveLeft),
            KeyCode::Char('l') if self.bindings.vim_navigation => Some(Action::MoveRight),
            KeyCode::Char('g') if self.bindings.vim_navigation => Some(Action::Home),
            KeyCode::Char('G') if self.bindings.vim_navigation => Some(Action::End),

            // Selection
            KeyCode::Enter => Some(Action::Select),
            KeyCode::Char(' ') => Some(Action::Select),

            // Back/Quit
            KeyCode::Esc => Some(Action::Back),
            KeyCode::Char('q') => Some(Action::Quit),

            // Actions
            KeyCode::Char('w') => Some(Action::SwitchWorktree),
            KeyCode::Char('W') => Some(Action::ManageWorktrees),
            KeyCode::Char('r') => Some(Action::RunWorkflow),
            KeyCode::Char('v') => Some(Action::ViewDocument),
            KeyCode::Char('e') => Some(Action::EditDocument),
            KeyCode::Char('n') => Some(Action::NewSpec),
            KeyCode::Char('d') => Some(Action::DeleteWorktree),
            KeyCode::Char('c') => Some(Action::CancelCommand),

            // Misc
            KeyCode::Char('?') => Some(Action::Help),
            KeyCode::F(5) => Some(Action::Refresh),

            _ => None,
        }
    }

    /// Handle key in insert mode
    fn handle_insert_key(&self, key: KeyEvent) -> Option<Action> {
        // In insert mode, Esc returns to normal mode
        if key.code == KeyCode::Esc {
            return Some(Action::Back);
        }

        // Ctrl+C also cancels
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Some(Action::Back);
        }

        // Other keys are handled by the text editor widget
        None
    }

    /// Handle key in command mode
    fn handle_command_key(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => Some(Action::Back),
            KeyCode::Enter => Some(Action::Select),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_navigation() {
        let handler = InputHandler::new(true);

        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_j, InputMode::Normal), Some(Action::MoveDown));

        let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_k, InputMode::Normal), Some(Action::MoveUp));
    }

    #[test]
    fn test_arrow_keys() {
        let handler = InputHandler::new(false); // vim disabled

        let key_up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_up, InputMode::Normal), Some(Action::MoveUp));

        let key_down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_down, InputMode::Normal), Some(Action::MoveDown));
    }

    #[test]
    fn test_action_keys() {
        let handler = InputHandler::new(true);

        let key_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_w, InputMode::Normal), Some(Action::SwitchWorktree));

        let key_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_r, InputMode::Normal), Some(Action::RunWorkflow));
    }

    #[test]
    fn test_quit_keys() {
        let handler = InputHandler::new(true);

        let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_q, InputMode::Normal), Some(Action::Quit));

        let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(handler.handle_key(key_esc, InputMode::Normal), Some(Action::Back));
    }
}
