//! Main layout rendering for the TUI.

use crate::app::{App, AppView, DocType};
use crate::ui::widgets::spec_list::SpecListWidget;
use crate::ui::widgets::output_panel::OutputPanelWidget;
use crate::ui::widgets::spec_detail::SpecDetailWidget;
use crate::ui::widgets::worktree_list::{WorktreeListWidget, ConfirmDialog};
use crate::ui::widgets::text_input::NewSpecDialog;
use crate::ui::widgets::help::HelpWidget;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

/// Draw the main application UI
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    match &app.view {
        AppView::Overview => draw_overview(frame, app, area),
        AppView::SpecDetail(id) => draw_spec_detail(frame, app, area, id),
        AppView::WorktreeManagement => draw_worktree_management(frame, app, area),
        AppView::DocumentView(doc_type) => draw_document_view(frame, app, area, doc_type),
        AppView::DocumentEdit(doc_type) => draw_document_edit(frame, app, area, doc_type),
        AppView::CommandOutput => draw_command_output(frame, app, area),
        AppView::WorkflowMenu => draw_workflow_menu(frame, app, area),
        AppView::NewSpec => draw_new_spec(frame, app, area),
        AppView::Help => draw_help(frame, app, area),
    }

    // Draw error message overlay if present
    if let Some(ref error) = app.error_message {
        draw_error_overlay(frame, error, area);
    }

    // Draw loading indicator if loading
    if app.is_loading {
        draw_loading_indicator_with_message(frame, area, app.loading_message.as_deref());
    }

    // Draw status message (non-blocking) if present and not loading
    if !app.is_loading {
        if let Some(ref msg) = app.loading_message {
            draw_status_message(frame, msg, area);
        }
    }
}

/// Draw a status message at the bottom of the screen
fn draw_status_message(frame: &mut Frame, message: &str, area: Rect) {
    // Create a small area at the bottom center
    let msg_area = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(4),
        width: area.width.saturating_sub(4).min(message.len() as u16 + 4),
        height: 3,
    };

    frame.render_widget(ratatui::widgets::Clear, msg_area);

    let status = Paragraph::new(message)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

    frame.render_widget(status, msg_area);
}

/// Draw the main overview with spec list
fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout: header, main content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header
    let header = Paragraph::new("spec-tui - Spec-Driven Development")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Main content - spec list
    if app.specs.is_empty() {
        let empty_msg = Paragraph::new("No specifications found.\n\nPress 'n' to create a new spec.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title("Specifications"))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, chunks[1]);
    } else {
        let spec_list = SpecListWidget::new(
            &app.specs,
            &app.worktrees,
            &app.worktree_statuses,
            app.selected_spec_index,
        );
        frame.render_widget(spec_list, chunks[1]);
    }

    // Footer with keybindings
    let footer_text = " j/k: Navigate | Enter: Select | w: Switch worktree | r: Run | n: New | q: Quit | ?: Help ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);
}

/// Draw spec detail view (placeholder)
fn draw_spec_detail(frame: &mut Frame, _app: &App, area: Rect, spec_id: &str) {
    let content = Paragraph::new(format!("Spec Detail: {}\n\n(Not yet implemented)", spec_id))
        .block(Block::default().borders(Borders::ALL).title("Specification Details"));
    frame.render_widget(content, area);
}

/// Draw worktree management view
fn draw_worktree_management(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout: header, worktree list, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Worktree list
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header
    let header = Paragraph::new("Worktree Management")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Worktree list
    if app.worktrees.is_empty() {
        let empty_msg = Paragraph::new("No worktrees found.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Worktrees "))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, chunks[1]);
    } else {
        let worktree_widget = WorktreeListWidget::new(
            &app.worktrees,
            &app.worktree_statuses,
            app.worktree_management_state.selected_index,
        )
        .sync_statuses(&app.worktree_sync_statuses);
        frame.render_widget(worktree_widget, chunks[1]);
    }

    // Footer with keybindings
    let footer_text = " j/k: Navigate | Enter: Switch | d: Delete | D: Force Delete | r: Refresh | q: Back ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);

    // Draw confirmation dialog if showing
    if app.worktree_management_state.showing_confirm {
        let popup_area = centered_rect(50, 30, area);

        let message = if let Some(ref path) = app.worktree_management_state.pending_delete {
            format!("Delete worktree at:\n{}\n\nThis cannot be undone!", path.display())
        } else {
            "Delete this worktree?".to_string()
        };

        let dialog = ConfirmDialog::new("Confirm Delete", &message)
            .yes_selected(app.worktree_management_state.confirm_yes_selected);
        frame.render_widget(dialog, popup_area);
    }
}

/// Draw document view with markdown highlighting
fn draw_document_view(frame: &mut Frame, app: &App, area: Rect, doc_type: &DocType) {
    // Create layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header with spec info
    let header_text = if let Some(spec) = app.selected_spec() {
        format!("{} - {}", doc_type_name(doc_type), spec.id.as_str())
    } else {
        doc_type_name(doc_type).to_string()
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Document content
    let content = app.document_content.as_deref().unwrap_or("No content");
    let title = doc_type_filename(doc_type);

    let visible_height = chunks[1].height.saturating_sub(2) as usize;

    let doc_widget = SpecDetailWidget::new(content, title)
        .scroll_offset(app.document_viewer_state.scroll_offset())
        .visible_height(visible_height);
    frame.render_widget(doc_widget, chunks[1]);

    // Footer with keybindings
    let footer_text = " q/Esc: Back | e: Edit | j/k: Scroll | 1-4: Switch doc ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);
}

/// Draw document edit view with editor widget
fn draw_document_edit(frame: &mut Frame, app: &App, area: Rect, doc_type: &DocType) {
    // Create layout: header, editor, status bar, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Editor
            Constraint::Length(1), // Status bar
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header with spec info
    let header_text = if let Some(spec) = app.selected_spec() {
        format!("Edit: {} - {}", doc_type_filename(doc_type), spec.id.as_str())
    } else {
        format!("Edit: {}", doc_type_filename(doc_type))
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Editor content
    if let Some(editor) = app.editor_state.editor() {
        frame.render_widget(editor, chunks[1]);

        // Status bar with cursor position and modified indicator
        let (line, col) = editor.cursor_position();
        let modified = if editor.is_modified() { "[Modified] " } else { "" };
        let status_text = format!(" {}Line {}, Col {} | {} lines ", modified, line + 1, col + 1, editor.line_count());

        let status_style = if editor.is_modified() {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };

        let status = Paragraph::new(status_text).style(status_style);
        frame.render_widget(status, chunks[2]);
    } else {
        let placeholder = Paragraph::new("No editor active")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(placeholder, chunks[1]);
    }

    // Footer with keybindings
    let footer_text = " Ctrl+S: Save | Esc: Cancel | Arrow keys: Navigate ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[3]);
}

/// Draw command output view with streaming output
fn draw_command_output(frame: &mut Frame, app: &App, area: Rect) {
    // Create layout: header, output, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with command info
            Constraint::Min(0),    // Output panel
            Constraint::Length(3), // Footer with keybindings
        ])
        .split(area);

    // Header with command info
    let header_text = if let Some(ref cmd) = app.active_command {
        let state_indicator = match &cmd.state {
            crate::domain::ExecutionState::Pending => "⏳ Pending",
            crate::domain::ExecutionState::Running { .. } => "▶ Running",
            crate::domain::ExecutionState::Completed { exit_code, .. } if *exit_code == 0 => "✓ Completed",
            crate::domain::ExecutionState::Completed { .. } => "✗ Completed (error)",
            crate::domain::ExecutionState::Failed { .. } => "✗ Failed",
            crate::domain::ExecutionState::Cancelled => "⊘ Cancelled",
        };
        format!("{} - {}", cmd.command_type.display_name(), state_indicator)
    } else {
        "No command".to_string()
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Output panel
    let output_widget = OutputPanelWidget::new(
        app.output_buffer.lines(),
        app.active_command.as_ref(),
    )
    .scroll_offset(app.output_buffer.scroll_offset())
    .auto_scroll(app.output_buffer.is_auto_scroll());
    frame.render_widget(output_widget, chunks[1]);

    // Footer with keybindings
    let footer_text = if app.is_command_running() {
        " c: Cancel | j/k: Scroll | G: Bottom "
    } else {
        " q/Esc: Back | j/k: Scroll | G: Bottom "
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);
}

/// Draw workflow command selection menu
fn draw_workflow_menu(frame: &mut Frame, app: &App, area: Rect) {
    // Draw the overview in the background
    draw_overview(frame, app, area);

    // Draw menu popup
    let popup_area = centered_rect(50, 40, area);
    frame.render_widget(ratatui::widgets::Clear, popup_area);

    // Get spec info for title
    let title = if let Some(spec) = app.selected_spec() {
        format!(" Run Workflow - {} ", spec.id.as_str())
    } else {
        " Run Workflow ".to_string()
    };

    // Build menu items
    let items: Vec<ListItem> = app
        .available_workflows
        .iter()
        .enumerate()
        .map(|(idx, cmd_type)| {
            let style = if idx == app.selected_workflow_index {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let text = format!("  {}  ", cmd_type.display_name());
            ListItem::new(text).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_workflow_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, popup_area, &mut state);

    // Footer hint at bottom of popup
    let hint_area = Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height - 2,
        width: popup_area.width,
        height: 1,
    };
    let hint = Paragraph::new(" Enter: Run | Esc: Cancel ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, hint_area);
}

/// Draw new spec creation dialog
fn draw_new_spec(frame: &mut Frame, app: &App, area: Rect) {
    // Draw the overview in the background
    draw_overview(frame, app, area);

    // Draw the dialog popup
    let popup_area = centered_rect(60, 40, area);

    let dialog = NewSpecDialog::new(&app.new_spec_input)
        .error(app.new_spec_error.as_deref());
    frame.render_widget(dialog, popup_area);
}

/// Draw help view showing all keybindings
fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    // We need a mutable reference to help_view_state, but we only have an immutable app reference.
    // The help widget will update state dimensions internally.
    // We'll clone the state and draw with it.
    let mut help_state = app.help_view_state.clone();
    let help_widget = HelpWidget::new(&mut help_state);
    frame.render_widget(help_widget, area);
}

/// Draw error overlay
fn draw_error_overlay(frame: &mut Frame, error: &str, area: Rect) {
    // Create a centered popup area
    let popup_area = centered_rect(60, 20, area);

    // Clear the area
    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let error_widget = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title("Error"),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(error_widget, popup_area);
}

/// Draw loading indicator with optional message
fn draw_loading_indicator(frame: &mut Frame, area: Rect) {
    draw_loading_indicator_with_message(frame, area, None);
}

/// Draw loading indicator with custom message
pub fn draw_loading_indicator_with_message(frame: &mut Frame, area: Rect, message: Option<&str>) {
    let popup_area = centered_rect(50, 5, area);

    frame.render_widget(ratatui::widgets::Clear, popup_area);

    let text = message.unwrap_or("Loading...");
    let loading = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    frame.render_widget(loading, popup_area);
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Get display name for a document type
fn doc_type_name(doc_type: &DocType) -> &'static str {
    match doc_type {
        DocType::Spec => "Specification",
        DocType::Plan => "Plan",
        DocType::Tasks => "Tasks",
        DocType::Research => "Research",
    }
}

/// Get filename for a document type
fn doc_type_filename(doc_type: &DocType) -> &'static str {
    match doc_type {
        DocType::Spec => "spec.md",
        DocType::Plan => "plan.md",
        DocType::Tasks => "tasks.md",
        DocType::Research => "research.md",
    }
}
