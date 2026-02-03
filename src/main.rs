mod actions;
mod app;
mod fs;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Focus, Mode};
use fs::scanner::scan_directories;
use ui::layout::render;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app
    let file_tree = scan_directories();
    let mut app = App::new(file_tree);

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Save state before exiting
    app.file_tree.save_state();

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C globally
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                }

                match app.mode {
                    Mode::Help => {
                        // Any key closes help
                        app.toggle_help();
                    }
                    Mode::Search => {
                        handle_search_input(app, key.code);
                    }
                    Mode::Normal => {
                        handle_normal_input(app, key.code, key.modifiers, terminal)?;
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_normal_input(
    app: &mut App,
    key: KeyCode,
    modifiers: KeyModifiers,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);

    match app.focus {
        Focus::Tree => match key {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('?') => app.toggle_help(),
            KeyCode::Char('/') => app.enter_search_mode(),
            KeyCode::Char('r') | KeyCode::Char('R') => app.refresh(),
            KeyCode::Char('.') => app.toggle_show_empty_dirs(),
            KeyCode::Char('c') => app.toggle_claude_only(),
            KeyCode::Up | KeyCode::Char('k') if ctrl => app.scroll_tree_up(),
            KeyCode::Down | KeyCode::Char('j') if ctrl => app.scroll_tree_down(),
            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
            KeyCode::Char(' ') => app.toggle_focus(),
            KeyCode::Tab => app.toggle_collapse(),
            KeyCode::Enter => {
                if let Some(path) = app.selected_file() {
                    if path.is_file() {
                        let path = path.clone();
                        // Restore terminal for editor
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;

                        // Open editor
                        if let Err(e) = actions::open_in_editor(&path) {
                            app.status_message = Some(e);
                        }

                        // Restore TUI
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                        terminal.hide_cursor()?;
                        terminal.clear()?;
                    }
                }
            }
            KeyCode::Esc => app.clear_search(),
            _ => {}
        },
        Focus::Preview => match key {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('?') => app.toggle_help(),
            KeyCode::Char('/') => app.enter_search_mode(),
            KeyCode::Char(' ') => app.toggle_focus(),
            KeyCode::Up | KeyCode::Char('k') => app.scroll_preview_up(),
            KeyCode::Down | KeyCode::Char('j') => app.scroll_preview_down(),
            KeyCode::Esc => app.toggle_focus(),
            _ => {}
        },
        Focus::Search => {}
    }
    Ok(())
}

fn handle_search_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.clear_search(),
        KeyCode::Enter => {
            app.exit_search_mode();
            if !app.filtered_indices.is_empty() {
                app.selected_index = 0;
            }
        }
        KeyCode::Backspace => app.pop_search_char(),
        KeyCode::Char(c) => app.push_search_char(c),
        _ => {}
    }
}
