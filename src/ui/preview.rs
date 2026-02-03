use std::fs;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Focus};

pub fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == Focus::Preview {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let selected_file = app.selected_file();

    let (title, content) = match selected_file {
        Some(path) if path.is_file() => {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            match fs::read_to_string(path) {
                Ok(content) => (format!(" {} ", filename), render_markdown(&content)),
                Err(e) => (
                    format!(" {} ", filename),
                    vec![Line::from(format!("Error reading file: {}", e))],
                ),
            }
        }
        Some(path) if path.is_dir() => {
            let dirname = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Directory".to_string());
            (
                format!(" {} ", dirname),
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Directory",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Select a markdown file to preview",
                        Style::default().fg(Color::DarkGray),
                    )),
                ],
            )
        }
        _ => (
            " Preview ".to_string(),
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  No file selected",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let scroll_indicator = if app.focus == Focus::Preview {
        format!(" [Scroll: {}] ", app.preview_scroll)
    } else {
        String::new()
    };

    let block = block.title_bottom(scroll_indicator);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn render_markdown(content: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(Color::Green),
            )));
            continue;
        }

        // Headers
        if line.starts_with("# ") {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if line.starts_with("## ") {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if line.starts_with("### ") {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if line.starts_with("#### ")
            || line.starts_with("##### ")
            || line.starts_with("###### ")
        {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        // Lists
        else if line.trim_start().starts_with("- ") || line.trim_start().starts_with("* ") {
            let indent = line.len() - line.trim_start().len();
            let spaces = " ".repeat(indent);
            let rest = line
                .trim_start()
                .strip_prefix("- ")
                .or_else(|| line.trim_start().strip_prefix("* "))
                .unwrap_or(line);
            lines.push(Line::from(vec![
                Span::raw(spaces),
                Span::styled("• ", Style::default().fg(Color::Cyan)),
                Span::raw(rest.to_string()),
            ]));
        }
        // Numbered lists
        else if line
            .trim_start()
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && line.trim_start().contains(". ")
        {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            )));
        }
        // Blockquotes
        else if line.starts_with("> ") {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));
        }
        // Horizontal rules
        else if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            lines.push(Line::from(Span::styled(
                "─".repeat(40),
                Style::default().fg(Color::DarkGray),
            )));
        }
        // Regular text with inline formatting
        else {
            lines.push(render_inline_markdown(line));
        }
    }

    lines
}

fn render_inline_markdown(line: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Bold **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 2;
            let mut bold_text = String::new();
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                bold_text.push(chars[i]);
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
            spans.push(Span::styled(
                bold_text,
                Style::default().add_modifier(Modifier::BOLD),
            ));
            continue;
        }

        // Inline code `text`
        if chars[i] == '`' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 1;
            let mut code_text = String::new();
            while i < chars.len() && chars[i] != '`' {
                code_text.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            spans.push(Span::styled(
                code_text,
                Style::default().fg(Color::Green).bg(Color::Rgb(30, 30, 30)),
            ));
            continue;
        }

        // Italic *text* (single asterisk)
        if chars[i] == '*' && (i + 1 >= chars.len() || chars[i + 1] != '*') {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            i += 1;
            let mut italic_text = String::new();
            while i < chars.len() && chars[i] != '*' {
                italic_text.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            spans.push(Span::styled(
                italic_text,
                Style::default().add_modifier(Modifier::ITALIC),
            ));
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    Line::from(spans)
}
