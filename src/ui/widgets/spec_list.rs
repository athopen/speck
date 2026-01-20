//! Spec list widget for displaying specifications in the overview.

use crate::domain::{Specification, WorkflowPhase, Worktree, WorktreeStatus};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Widget for displaying a list of specifications
pub struct SpecListWidget<'a> {
    specs: &'a [Specification],
    worktrees: &'a [Worktree],
    worktree_statuses: &'a HashMap<PathBuf, WorktreeStatus>,
    selected_index: usize,
}

impl<'a> SpecListWidget<'a> {
    /// Create a new spec list widget
    pub fn new(
        specs: &'a [Specification],
        worktrees: &'a [Worktree],
        worktree_statuses: &'a HashMap<PathBuf, WorktreeStatus>,
        selected_index: usize,
    ) -> Self {
        Self {
            specs,
            worktrees,
            worktree_statuses,
            selected_index,
        }
    }

    /// Get phase indicator character
    fn phase_indicator(phase: &WorkflowPhase) -> &'static str {
        match phase {
            WorkflowPhase::Specify => "○",   // Empty circle - needs spec
            WorkflowPhase::Clarify => "◐",   // Half circle - needs clarification
            WorkflowPhase::Tasks => "◑",     // Half circle - needs tasks
            WorkflowPhase::Implement => "●", // Full circle - ready to implement
        }
    }

    /// Get phase color
    fn phase_color(phase: &WorkflowPhase) -> Color {
        match phase {
            WorkflowPhase::Specify => Color::DarkGray,
            WorkflowPhase::Clarify => Color::Yellow,
            WorkflowPhase::Tasks => Color::Blue,
            WorkflowPhase::Implement => Color::Green,
        }
    }

    /// Find worktree for a spec
    fn find_worktree(&self, spec: &Specification) -> Option<&Worktree> {
        self.worktrees
            .iter()
            .find(|w| w.branch.contains(&spec.branch) || spec.branch.contains(&w.branch))
    }

    /// Get worktree status indicator
    fn worktree_status_indicator(&self, worktree: &Worktree) -> &'static str {
        match self.worktree_statuses.get(&worktree.path) {
            Some(WorktreeStatus::Clean) => "✓",          // Clean
            Some(WorktreeStatus::Dirty { .. }) => "●",   // Has changes
            Some(WorktreeStatus::Detached) => "!",       // Detached HEAD
            Some(WorktreeStatus::Unknown) | None => "?", // Unknown
        }
    }

    /// Get worktree status color
    fn worktree_status_color(&self, worktree: &Worktree) -> Color {
        match self.worktree_statuses.get(&worktree.path) {
            Some(WorktreeStatus::Clean) => Color::Green,
            Some(WorktreeStatus::Dirty { .. }) => Color::Yellow,
            Some(WorktreeStatus::Detached) => Color::Magenta,
            Some(WorktreeStatus::Unknown) | None => Color::DarkGray,
        }
    }

    /// Build list items from specs
    fn build_items(&self) -> Vec<ListItem<'a>> {
        self.specs
            .iter()
            .enumerate()
            .map(|(idx, spec)| {
                let phase = &spec.phase;
                let indicator = Self::phase_indicator(phase);
                let color = Self::phase_color(phase);

                // Check for worktree and its status
                let worktree_info = if let Some(wt) = self.find_worktree(spec) {
                    let status_indicator = self.worktree_status_indicator(wt);
                    format!(" [wt {}]", status_indicator)
                } else {
                    String::new()
                };

                // Format: [indicator] spec-id: name [phase] [worktree status]
                let line = format!(
                    "{} {}: {} {}{}",
                    indicator,
                    spec.id.as_str(),
                    spec.name,
                    phase.badge(),
                    worktree_info
                );

                let style = if idx == self.selected_index {
                    Style::default()
                        .fg(Color::White)
                        .bg(color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                };

                ListItem::new(line).style(style)
            })
            .collect()
    }
}

impl Widget for SpecListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items = self.build_items();

        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Specifications"),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        // Use StatefulWidget render
        StatefulWidget::render(list, area, buf, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_indicators() {
        assert_eq!(
            SpecListWidget::phase_indicator(&WorkflowPhase::Specify),
            "○"
        );
        assert_eq!(
            SpecListWidget::phase_indicator(&WorkflowPhase::Implement),
            "●"
        );
    }

    #[test]
    fn test_empty_list() {
        let specs: Vec<Specification> = vec![];
        let worktrees: Vec<Worktree> = vec![];
        let statuses: HashMap<PathBuf, WorktreeStatus> = HashMap::new();
        let widget = SpecListWidget::new(&specs, &worktrees, &statuses, 0);
        let items = widget.build_items();
        assert!(items.is_empty());
    }
}
