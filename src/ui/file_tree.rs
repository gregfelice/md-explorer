use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, Focus};

pub fn render_file_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    // Update tree height for scroll calculations (subtract 2 for borders)
    app.tree_height = area.height.saturating_sub(2) as usize;
    let border_color = if app.focus == Focus::Tree {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Files ");

    let flat_list = app.file_tree.flat_list();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(display_idx, &actual_idx)| {
            let (path, depth) = &flat_list[actual_idx];

            let is_selected = display_idx == app.selected_index;
            let is_dir = path.is_dir();

            // Create display name
            let display_name = if app.file_tree.roots.contains(path) {
                // Show root directories with ~ prefix
                format!(
                    "~{}",
                    path.strip_prefix(&home).unwrap_or(path).to_string_lossy()
                )
            } else {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string())
            };

            // Build tree prefix
            let indent = "  ".repeat(*depth);
            let prefix = if app.file_tree.roots.contains(path) {
                ""
            } else {
                "├── "
            };

            // Show collapse/expand indicator for directories
            let icon = if is_dir {
                let has_children = app.file_tree.has_children(path);
                let is_collapsed = app.file_tree.is_collapsed(path);
                if has_children {
                    if is_collapsed {
                        "▶ 📁 "
                    } else {
                        "▼ 📁 "
                    }
                } else {
                    "  📁 "
                }
            } else {
                "  📄 "
            };

            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(40, 40, 60))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_dir {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(format!("{}{}", icon, display_name), style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 40, 60))
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    *state.offset_mut() = app.tree_scroll;

    frame.render_stateful_widget(list, area, &mut state);
}
