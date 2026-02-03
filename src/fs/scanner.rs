use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use ignore::WalkBuilder;

#[derive(Debug, Clone)]
pub struct FileTree {
    pub roots: Vec<PathBuf>,
    pub entries: BTreeMap<PathBuf, Vec<PathBuf>>,
    flat_cache: Vec<(PathBuf, usize)>,
    pub collapsed: HashSet<PathBuf>,
    pub dirs_with_md: HashSet<PathBuf>,
    pub dirs_with_claude_md: HashSet<PathBuf>,
    pub show_empty_dirs: bool,
    pub claude_only: bool,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            entries: BTreeMap::new(),
            flat_cache: Vec::new(),
            collapsed: HashSet::new(),
            dirs_with_md: HashSet::new(),
            dirs_with_claude_md: HashSet::new(),
            show_empty_dirs: false,
            claude_only: false,
        }
    }

    pub fn flat_list(&self) -> &[(PathBuf, usize)] {
        &self.flat_cache
    }

    pub fn rebuild_flat_cache(&mut self) {
        self.build_flat_cache();
    }

    fn build_flat_cache(&mut self) {
        self.flat_cache.clear();
        for root in &self.roots.clone() {
            self.flatten_dir(root, 0);
        }
    }

    fn flatten_dir(&mut self, dir: &PathBuf, depth: usize) {
        self.flat_cache.push((dir.clone(), depth));

        // Skip children if this directory is collapsed
        if self.collapsed.contains(dir) {
            return;
        }

        if let Some(children) = self.entries.get(dir).cloned() {
            let mut dirs: Vec<_> = children.iter().filter(|p| p.is_dir()).collect();
            let mut files: Vec<_> = children.iter().filter(|p| p.is_file()).collect();

            dirs.sort();
            files.sort();

            // Show files first, then subdirectories
            for file in files {
                // When claude_only is true, only show CLAUDE.md files
                if self.claude_only {
                    if let Some(name) = file.file_name() {
                        if name != "CLAUDE.md" {
                            continue;
                        }
                    }
                }
                self.flat_cache.push((file.clone(), depth + 1));
            }

            for child_dir in dirs {
                // Skip directories based on filter mode
                if self.claude_only {
                    // In claude_only mode, only show dirs with CLAUDE.md
                    if !self.dirs_with_claude_md.contains(child_dir) {
                        continue;
                    }
                } else if !self.show_empty_dirs && !self.dirs_with_md.contains(child_dir) {
                    // Normal mode: skip directories without md files unless show_empty_dirs is enabled
                    continue;
                }
                self.flatten_dir(child_dir, depth + 1);
            }
        }
    }

    pub fn toggle_collapsed(&mut self, path: &PathBuf) -> bool {
        if self.collapsed.contains(path) {
            self.collapsed.remove(path);
            false // Now expanded
        } else {
            self.collapsed.insert(path.clone());
            true // Now collapsed
        }
    }

    pub fn is_collapsed(&self, path: &PathBuf) -> bool {
        self.collapsed.contains(path)
    }

    pub fn has_children(&self, path: &PathBuf) -> bool {
        self.entries
            .get(path)
            .map(|children| {
                children.iter().any(|c| {
                    if c.is_file() {
                        if self.claude_only {
                            c.file_name().map(|n| n == "CLAUDE.md").unwrap_or(false)
                        } else {
                            true
                        }
                    } else if self.claude_only {
                        self.dirs_with_claude_md.contains(c)
                    } else if self.show_empty_dirs {
                        true
                    } else {
                        self.dirs_with_md.contains(c)
                    }
                })
            })
            .unwrap_or(false)
    }

    pub fn toggle_show_empty_dirs(&mut self) -> bool {
        self.show_empty_dirs = !self.show_empty_dirs;
        self.show_empty_dirs
    }

    pub fn toggle_claude_only(&mut self) -> bool {
        self.claude_only = !self.claude_only;
        self.claude_only
    }

    fn state_file_path() -> Option<PathBuf> {
        dirs::state_dir()
            .or_else(dirs::data_local_dir)
            .map(|p| p.join("md-explorer").join("state"))
    }

    pub fn load_state(&mut self) {
        let Some(state_path) = Self::state_file_path() else {
            return;
        };

        let Ok(file) = fs::File::open(&state_path) else {
            return;
        };

        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("collapsed:") {
                let path_str = line.strip_prefix("collapsed:").unwrap();
                let path = PathBuf::from(path_str);
                if path.exists() {
                    self.collapsed.insert(path);
                }
            } else if line == "show_empty_dirs:true" {
                self.show_empty_dirs = true;
            } else if line == "claude_only:true" {
                self.claude_only = true;
            }
        }
    }

    pub fn save_state(&self) {
        let Some(state_path) = Self::state_file_path() else {
            return;
        };

        if let Some(parent) = state_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let Ok(mut file) = fs::File::create(&state_path) else {
            return;
        };

        for path in &self.collapsed {
            let _ = writeln!(file, "collapsed:{}", path.display());
        }

        if self.show_empty_dirs {
            let _ = writeln!(file, "show_empty_dirs:true");
        }

        if self.claude_only {
            let _ = writeln!(file, "claude_only:true");
        }
    }
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}

pub fn scan_directories() -> FileTree {
    let mut tree = FileTree::new();

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let operations = home.join("operations");
    let development = home.join("development");

    let roots: Vec<PathBuf> = [operations, development]
        .into_iter()
        .filter(|p| p.exists())
        .collect();

    tree.roots = roots.clone();

    for root in &roots {
        scan_directory(root, &mut tree);
    }

    // Ensure roots are always marked as having md content if they have any entries
    for root in &roots {
        if tree
            .entries
            .get(root)
            .map(|c| !c.is_empty())
            .unwrap_or(false)
        {
            tree.dirs_with_md.insert(root.clone());
        }
    }

    tree.load_state();
    tree.build_flat_cache();
    tree
}

fn scan_directory(root: &PathBuf, tree: &mut FileTree) {
    let walker = WalkBuilder::new(root)
        .hidden(true)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "node_modules"
                    | "vendor"
                    | ".git"
                    | "target"
                    | "__pycache__"
                    | "venv"
                    | ".venv"
                    | ".cache"
                    | "dist"
                    | "build"
            )
        })
        .build();

    let mut all_dirs: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    all_dirs.insert(root.clone(), Vec::new());

    let mut md_files: Vec<PathBuf> = Vec::new();
    let mut claude_md_files: Vec<PathBuf> = Vec::new();

    for entry in walker.flatten() {
        let path = entry.path().to_path_buf();

        if path == *root {
            continue;
        }

        let is_md = path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        let is_dir = path.is_dir();

        if !is_md && !is_dir {
            continue;
        }

        if let Some(parent) = path.parent() {
            let parent_path = parent.to_path_buf();
            all_dirs.entry(parent_path).or_default().push(path.clone());
        }

        if is_dir {
            all_dirs.entry(path.clone()).or_default();
        }

        if is_md {
            md_files.push(path.clone());
            // Track CLAUDE.md files separately
            if path.file_name().map(|n| n == "CLAUDE.md").unwrap_or(false) {
                claude_md_files.push(path);
            }
        }
    }

    // Mark all ancestor directories of md files as having md content
    for md_path in &md_files {
        let mut current = md_path.parent();
        while let Some(parent) = current {
            let parent_path = parent.to_path_buf();
            if tree.dirs_with_md.contains(&parent_path) {
                break; // Already marked, ancestors are too
            }
            tree.dirs_with_md.insert(parent_path.clone());
            current = parent.parent();
        }
    }

    // Mark all ancestor directories of CLAUDE.md files
    for claude_path in &claude_md_files {
        let mut current = claude_path.parent();
        while let Some(parent) = current {
            let parent_path = parent.to_path_buf();
            if tree.dirs_with_claude_md.contains(&parent_path) {
                break; // Already marked, ancestors are too
            }
            tree.dirs_with_claude_md.insert(parent_path.clone());
            current = parent.parent();
        }
    }

    tree.entries.extend(all_dirs);
}
