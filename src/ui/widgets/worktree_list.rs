//! Worktree list widget for managing git worktrees.

use crate::domain::{Worktree, WorktreeStatus, WorktreeSyncStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Widget for displaying and managing worktrees
pub struct WorktreeListWidget<'a> {
    /// Worktrees to display
    worktrees: &'a [Worktree],
    /// Status cache for worktrees
    statuses: &'a HashMap<PathBuf, WorktreeStatus>,
    /// Sync status cache for worktrees
    sync_statuses: &'a HashMap<String, WorktreeSyncStatus>,
    /// Currently selected index
    selected_index: usize,
    /// Currently active worktree path (if any)
    active_worktree: Option<&'a PathBuf>,
}

impl<'a> WorktreeListWidget<'a> {
    /// Create a new worktree list widget
    pub fn new(
        worktrees: &'a [Worktree],
        statuses: &'a HashMap<PathBuf, WorktreeStatus>,
        selected_index: usize,
    ) -> Self {
        // Use a static empty hashmap for default
        static EMPTY_SYNC: std::sync::OnceLock<HashMap<String, WorktreeSyncStatus>> = std::sync::OnceLock::new();
        let empty = EMPTY_SYNC.get_or_init(HashMap::new);

        Self {
            worktrees,
            statuses,
            sync_statuses: empty,
            selected_index,
            active_worktree: None,
        }
    }

    /// Set sync status cache
    pub fn sync_statuses(mut self, sync_statuses: &'a HashMap<String, WorktreeSyncStatus>) -> Self {
        self.sync_statuses = sync_statuses;
        self
    }

    /// Set the active worktree
    pub fn active_worktree(mut self, path: Option<&'a PathBuf>) -> Self {
        self.active_worktree = path;
        self
    }

    /// Format a worktree entry for display
    fn format_worktree(&self, worktree: &Worktree, is_selected: bool) -> ListItem<'a> {
        let status = self.statuses.get(&worktree.path);
        let sync_status = self.sync_statuses.get(&worktree.branch);

        // Build status indicators
        let mut indicators = Vec::new();

        // Main worktree indicator
        if worktree.is_main {
            indicators.push(Span::styled("[main]", Style::default().fg(Color::Magenta)));
            indicators.push(Span::raw(" "));
        }

        // Active indicator
        let is_active = self.active_worktree.map_or(false, |p| p == &worktree.path);
        if is_active {
            indicators.push(Span::styled("● ", Style::default().fg(Color::Green)));
        } else {
            indicators.push(Span::raw("  "));
        }

        // Branch name
        let branch_style = if is_selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };
        indicators.push(Span::styled(worktree.branch.clone(), branch_style));

        // Working directory status
        if let Some(status) = status {
            let status_text = match status {
                WorktreeStatus::Clean => Span::styled(" ✓", Style::default().fg(Color::Green)),
                WorktreeStatus::Dirty { modified, staged, untracked } => {
                    let parts: Vec<String> = [
                        if *modified > 0 { Some(format!("~{}", modified)) } else { None },
                        if *staged > 0 { Some(format!("+{}", staged)) } else { None },
                        if *untracked > 0 { Some(format!("?{}", untracked)) } else { None },
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
                    Span::styled(format!(" [{}]", parts.join(" ")), Style::default().fg(Color::Yellow))
                }
                WorktreeStatus::Detached => Span::styled(" (detached)", Style::default().fg(Color::Red)),
                WorktreeStatus::Unknown => Span::styled(" ?", Style::default().fg(Color::DarkGray)),
            };
            indicators.push(status_text);
        }

        // Sync status (ahead/behind remote)
        if let Some(sync) = sync_status {
            if sync.remote_exists {
                let sync_text = match (sync.ahead, sync.behind) {
                    (0, 0) => Span::styled(" ≡", Style::default().fg(Color::Green)),
                    (a, 0) => Span::styled(format!(" ↑{}", a), Style::default().fg(Color::Blue)),
                    (0, b) => Span::styled(format!(" ↓{}", b), Style::default().fg(Color::Red)),
                    (a, b) => Span::styled(format!(" ↑{}↓{}", a, b), Style::default().fg(Color::Yellow)),
                };
                indicators.push(sync_text);
            }
        }

        // Path (abbreviated)
        let path_str = worktree.path.to_string_lossy();
        let abbreviated_path = if path_str.len() > 40 {
            format!("...{}", &path_str[path_str.len() - 37..])
        } else {
            path_str.to_string()
        };
        indicators.push(Span::raw("  "));
        indicators.push(Span::styled(abbreviated_path, Style::default().fg(Color::DarkGray)));

        let line = Line::from(indicators);

        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        ListItem::new(line).style(style)
    }
}

impl Widget for WorktreeListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Worktrees ");
        let inner = block.inner(area);
        block.render(area, buf);

        // Render items manually
        for (idx, wt) in self.worktrees.iter().enumerate() {
            if idx >= inner.height as usize {
                break;
            }

            let is_selected = idx == self.selected_index;
            let y = inner.y + idx as u16;

            // Build the line content
            let status = self.statuses.get(&wt.path);
            let sync_status = self.sync_statuses.get(&wt.branch);

            // Selection indicator
            let prefix = if is_selected { "> " } else { "  " };
            let prefix_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut x = inner.x;

            // Selection indicator
            buf.set_string(x, y, prefix, prefix_style);
            x += 2;

            // Main worktree indicator
            if wt.is_main {
                buf.set_string(x, y, "[main] ", Style::default().fg(Color::Magenta));
                x += 7;
            }

            // Branch name
            let branch_style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let branch_display = if wt.branch.len() > 30 {
                format!("{}...", &wt.branch[..27])
            } else {
                wt.branch.clone()
            };
            buf.set_string(x, y, &branch_display, branch_style);
            x += branch_display.len() as u16 + 1;

            // Working directory status
            if let Some(status) = status {
                let (status_text, style) = match status {
                    WorktreeStatus::Clean => ("✓".to_string(), Style::default().fg(Color::Green)),
                    WorktreeStatus::Dirty { modified, staged, untracked } => {
                        let parts: Vec<String> = [
                            if *modified > 0 { Some(format!("~{}", modified)) } else { None },
                            if *staged > 0 { Some(format!("+{}", staged)) } else { None },
                            if *untracked > 0 { Some(format!("?{}", untracked)) } else { None },
                        ]
                        .into_iter()
                        .flatten()
                        .collect();
                        (format!("[{}]", parts.join(" ")), Style::default().fg(Color::Yellow))
                    },
                    WorktreeStatus::Detached => ("(detached)".to_string(), Style::default().fg(Color::Red)),
                    WorktreeStatus::Unknown => ("?".to_string(), Style::default().fg(Color::DarkGray)),
                };
                buf.set_string(x, y, &status_text, style);
                x += status_text.len() as u16 + 1;
            }

            // Sync status
            if let Some(sync) = sync_status {
                if sync.remote_exists {
                    let (sync_text, style) = match (sync.ahead, sync.behind) {
                        (0, 0) => ("≡".to_string(), Style::default().fg(Color::Green)),
                        (a, 0) => (format!("↑{}", a), Style::default().fg(Color::Blue)),
                        (0, b) => (format!("↓{}", b), Style::default().fg(Color::Red)),
                        (a, b) => (format!("↑{}↓{}", a, b), Style::default().fg(Color::Yellow)),
                    };
                    buf.set_string(x, y, &sync_text, style);
                    x += sync_text.len() as u16 + 1;
                }
            }

            // Path (abbreviated)
            let remaining_width = inner.width.saturating_sub(x - inner.x + 2) as usize;
            if remaining_width > 5 {
                let path_str = wt.path.to_string_lossy();
                let abbreviated_path = if path_str.len() > remaining_width {
                    format!("...{}", &path_str[path_str.len() - (remaining_width - 3)..])
                } else {
                    path_str.to_string()
                };
                buf.set_string(x + 1, y, &abbreviated_path, Style::default().fg(Color::DarkGray));
            }
        }
    }
}

/// Confirmation dialog widget
pub struct ConfirmDialog<'a> {
    /// Title of the dialog
    title: &'a str,
    /// Message to display
    message: &'a str,
    /// Whether "Yes" is selected (vs "No")
    yes_selected: bool,
}

impl<'a> ConfirmDialog<'a> {
    /// Create a new confirmation dialog
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            yes_selected: false,
        }
    }

    /// Set whether yes is selected
    pub fn yes_selected(mut self, selected: bool) -> Self {
        self.yes_selected = selected;
        self
    }
}

impl Widget for ConfirmDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        Clear.render(area, buf);

        // Render border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" {} ", self.title));
        let inner = block.inner(area);
        block.render(area, buf);

        // Render message
        let message_area = Rect {
            x: inner.x + 1,
            y: inner.y + 1,
            width: inner.width.saturating_sub(2),
            height: inner.height.saturating_sub(4),
        };
        let message = Paragraph::new(self.message)
            .style(Style::default())
            .wrap(ratatui::widgets::Wrap { trim: true });
        message.render(message_area, buf);

        // Render buttons
        let buttons_y = inner.y + inner.height.saturating_sub(2);
        let button_width = 10u16;
        let total_buttons_width = button_width * 2 + 4;
        let start_x = inner.x + (inner.width.saturating_sub(total_buttons_width)) / 2;

        // "Yes" button
        let yes_style = if self.yes_selected {
            Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let yes_text = if self.yes_selected { "[ Yes ]" } else { "  Yes  " };
        buf.set_string(start_x, buttons_y, yes_text, yes_style);

        // "No" button
        let no_style = if !self.yes_selected {
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let no_text = if !self.yes_selected { "[  No  ]" } else { "   No   " };
        buf.set_string(start_x + button_width + 2, buttons_y, no_text, no_style);
    }
}

/// State for worktree management view
#[derive(Debug, Default)]
pub struct WorktreeManagementState {
    /// Selected worktree index
    pub selected_index: usize,
    /// Whether confirmation dialog is showing
    pub showing_confirm: bool,
    /// Confirmation dialog selection (true = yes)
    pub confirm_yes_selected: bool,
    /// Worktree pending deletion
    pub pending_delete: Option<PathBuf>,
}

impl WorktreeManagementState {
    /// Create new state
    pub fn new() -> Self {
        Self::default()
    }

    /// Select previous worktree
    pub fn select_previous(&mut self, count: usize) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Select next worktree
    pub fn select_next(&mut self, count: usize) {
        if self.selected_index < count.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Show deletion confirmation for a worktree
    pub fn request_delete(&mut self, path: PathBuf) {
        self.pending_delete = Some(path);
        self.showing_confirm = true;
        self.confirm_yes_selected = false; // Default to "No" for safety
    }

    /// Toggle confirmation selection
    pub fn toggle_confirm_selection(&mut self) {
        self.confirm_yes_selected = !self.confirm_yes_selected;
    }

    /// Cancel confirmation dialog
    pub fn cancel_confirm(&mut self) {
        self.showing_confirm = false;
        self.pending_delete = None;
        self.confirm_yes_selected = false;
    }

    /// Confirm deletion (returns path to delete if confirmed)
    pub fn confirm_delete(&mut self) -> Option<PathBuf> {
        if self.confirm_yes_selected {
            let path = self.pending_delete.take();
            self.showing_confirm = false;
            self.confirm_yes_selected = false;
            path
        } else {
            self.cancel_confirm();
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_management_state() {
        let mut state = WorktreeManagementState::new();

        state.select_next(5);
        assert_eq!(state.selected_index, 1);

        state.select_previous(5);
        assert_eq!(state.selected_index, 0);

        // Can't go below 0
        state.select_previous(5);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_confirm_dialog_state() {
        let mut state = WorktreeManagementState::new();

        state.request_delete(PathBuf::from("/test/path"));
        assert!(state.showing_confirm);
        assert!(!state.confirm_yes_selected);
        assert!(state.pending_delete.is_some());

        state.toggle_confirm_selection();
        assert!(state.confirm_yes_selected);

        let deleted = state.confirm_delete();
        assert!(deleted.is_some());
        assert!(!state.showing_confirm);
    }
}
