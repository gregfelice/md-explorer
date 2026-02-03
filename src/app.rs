use std::path::PathBuf;

use crate::fs::scanner::FileTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Preview,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
    Help,
}

pub struct App {
    pub file_tree: FileTree,
    pub selected_index: usize,
    pub tree_scroll: usize,
    pub tree_height: usize,
    pub preview_scroll: u16,
    pub focus: Focus,
    pub mode: Mode,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
    pub should_quit: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(file_tree: FileTree) -> Self {
        let total_items = file_tree.flat_list().len();
        let filtered_indices: Vec<usize> = (0..total_items).collect();

        Self {
            file_tree,
            selected_index: 0,
            tree_scroll: 0,
            tree_height: 20, // Will be updated by render
            preview_scroll: 0,
            focus: Focus::Tree,
            mode: Mode::Normal,
            search_query: String::new(),
            filtered_indices,
            should_quit: false,
            status_message: None,
        }
    }

    pub fn selected_file(&self) -> Option<&PathBuf> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let actual_index = self.filtered_indices.get(self.selected_index)?;
        let flat = self.file_tree.flat_list();
        flat.get(*actual_index).map(|(path, _)| path)
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.preview_scroll = 0;
            // Scroll up if selection goes above visible area
            if self.selected_index < self.tree_scroll {
                self.tree_scroll = self.selected_index;
            }
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered_indices.is_empty()
            && self.selected_index < self.filtered_indices.len() - 1
        {
            self.selected_index += 1;
            self.preview_scroll = 0;
            // Scroll down if selection goes below visible area
            let visible_end = self.tree_scroll + self.tree_height.saturating_sub(1);
            if self.selected_index > visible_end {
                self.tree_scroll = self
                    .selected_index
                    .saturating_sub(self.tree_height.saturating_sub(1));
            }
        }
    }

    pub fn scroll_tree_up(&mut self) {
        // Scroll pane up, move selection to stay at same visual position
        if self.tree_scroll > 0 {
            self.tree_scroll -= 1;
            // Move selection up to maintain visual position
            if self.selected_index > 0 {
                self.selected_index -= 1;
                self.preview_scroll = 0;
            }
        }
    }

    pub fn scroll_tree_down(&mut self) {
        let max_scroll = self.filtered_indices.len().saturating_sub(self.tree_height);
        if self.tree_scroll < max_scroll {
            self.tree_scroll += 1;
            // Move selection down to maintain visual position
            if self.selected_index < self.filtered_indices.len() - 1 {
                self.selected_index += 1;
                self.preview_scroll = 0;
            }
        }
    }

    pub fn scroll_preview_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll -= 1;
        }
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll += 1;
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tree => Focus::Preview,
            Focus::Preview => Focus::Tree,
            Focus::Search => Focus::Tree,
        };
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = Mode::Search;
        self.focus = Focus::Search;
        self.search_query.clear();
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = Mode::Normal;
        self.focus = Focus::Tree;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_filter();
        self.exit_search_mode();
    }

    pub fn update_filter(&mut self) {
        use crate::fs::filter::fuzzy_filter;

        let flat = self.file_tree.flat_list();
        if self.search_query.is_empty() {
            self.filtered_indices = (0..flat.len()).collect();
        } else {
            self.filtered_indices = fuzzy_filter(flat, &self.search_query);
        }

        if !self.filtered_indices.is_empty() && self.selected_index >= self.filtered_indices.len() {
            self.selected_index = 0;
        }
    }

    pub fn push_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }

    pub fn toggle_help(&mut self) {
        self.mode = match self.mode {
            Mode::Help => Mode::Normal,
            _ => Mode::Help,
        };
    }

    pub fn refresh(&mut self) {
        use crate::fs::scanner::scan_directories;

        self.file_tree = scan_directories();
        self.update_filter();
        self.status_message = Some("Refreshed file list".to_string());
    }

    pub fn toggle_collapse(&mut self) {
        if let Some(path) = self.selected_file().cloned() {
            if path.is_dir() {
                self.file_tree.toggle_collapsed(&path);
                self.file_tree.rebuild_flat_cache();
                self.update_filter();
            }
        }
    }

    pub fn toggle_show_empty_dirs(&mut self) {
        let showing = self.file_tree.toggle_show_empty_dirs();
        self.file_tree.rebuild_flat_cache();
        let count = self.file_tree.flat_list().len();
        self.update_filter();
        self.status_message = Some(if showing {
            format!("Showing all directories ({} items)", count)
        } else {
            format!("Hiding empty directories ({} items)", count)
        });
    }

    pub fn toggle_claude_only(&mut self) {
        let claude_only = self.file_tree.toggle_claude_only();
        self.file_tree.rebuild_flat_cache();
        let count = self.file_tree.flat_list().len();
        self.update_filter();
        // Reset selection if it's now out of bounds
        if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = 0;
        }
        self.status_message = Some(if claude_only {
            format!("Showing CLAUDE.md only ({} items)", count)
        } else {
            format!("Showing all markdown files ({} items)", count)
        });
    }
}
