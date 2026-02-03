use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Mode};
use crate::ui::file_tree::render_file_tree;
use crate::ui::preview::render_preview;
use crate::ui::search::render_search_bar;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    render_search_bar(frame, app, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    if app.mode == Mode::Help {
        render_help_popup(frame);
    }
}

fn render_main_content(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // File tree
            Constraint::Percentage(65), // Preview
        ])
        .split(area);

    render_file_tree(frame, app, chunks[0]);
    render_preview(frame, app, chunks[1]);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let focus_indicator = match app.focus {
        Focus::Tree => "[Tree]",
        Focus::Preview => "[Preview]",
        Focus::Search => "[Search]",
    };

    let help_text = vec![
        Span::styled("↑↓/jk", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(" Expand/Collapse  "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" Open  "),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::raw(" Focus  "),
        Span::styled("/", Style::default().fg(Color::Cyan)),
        Span::raw(" Search  "),
        Span::styled("?", Style::default().fg(Color::Cyan)),
        Span::raw(" Help  "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(" Quit  "),
        Span::styled(focus_indicator, Style::default().fg(Color::Yellow)),
    ];

    let status =
        Paragraph::new(Line::from(help_text)).block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}

fn render_help_popup(frame: &mut Frame) {
    let area = frame.area();

    // Center the popup
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 21.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    let clear = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "MD Explorer - Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑/k      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move selection up"),
        ]),
        Line::from(vec![
            Span::styled("↓/j      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move selection down"),
        ]),
        Line::from(vec![
            Span::styled("Tab      ", Style::default().fg(Color::Yellow)),
            Span::raw("Expand/collapse directory"),
        ]),
        Line::from(vec![
            Span::styled(".        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle empty directories"),
        ]),
        Line::from(vec![
            Span::styled("c        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle CLAUDE.md only"),
        ]),
        Line::from(vec![
            Span::styled("Enter    ", Style::default().fg(Color::Yellow)),
            Span::raw("Open file in $EDITOR"),
        ]),
        Line::from(vec![
            Span::styled("Space    ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle focus (tree/preview)"),
        ]),
        Line::from(vec![
            Span::styled("/        ", Style::default().fg(Color::Yellow)),
            Span::raw("Start search/filter"),
        ]),
        Line::from(vec![
            Span::styled("Esc      ", Style::default().fg(Color::Yellow)),
            Span::raw("Clear search / exit mode"),
        ]),
        Line::from(vec![
            Span::styled("r/R      ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh file list"),
        ]),
        Line::from(vec![
            Span::styled("?        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help screen"),
        ]),
        Line::from(vec![
            Span::styled("q        ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help_paragraph = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Help "),
    );

    frame.render_widget(help_paragraph, popup_area);
}
