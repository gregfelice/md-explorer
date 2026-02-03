use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};

pub fn render_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let is_searching = app.mode == Mode::Search;

    let border_color = if is_searching {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let search_label = if is_searching {
        Span::styled("/", Style::default().fg(Color::Cyan))
    } else {
        Span::styled("[/]", Style::default().fg(Color::DarkGray))
    };

    let search_text = if app.search_query.is_empty() && !is_searching {
        Span::styled(" Search...", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            format!(" {}", app.search_query),
            Style::default().fg(Color::White),
        )
    };

    let cursor = if is_searching {
        Span::styled("█", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    let result_count = if !app.search_query.is_empty() {
        let file_count = app
            .filtered_indices
            .iter()
            .filter(|&&idx| {
                let flat = app.file_tree.flat_list();
                flat.get(idx).map(|(p, _)| p.is_file()).unwrap_or(false)
            })
            .count();
        Span::styled(
            format!("  [{} matches]", file_count),
            Style::default().fg(Color::Yellow),
        )
    } else {
        Span::raw("")
    };

    let help_hint = Span::styled("  [?] Help", Style::default().fg(Color::DarkGray));

    let line = Line::from(vec![
        search_label,
        search_text,
        cursor,
        result_count,
        Span::raw("  "),
        help_hint,
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" MD Explorer ");

    let paragraph = Paragraph::new(line).block(block);

    frame.render_widget(paragraph, area);
}
