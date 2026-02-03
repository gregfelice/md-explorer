use std::path::PathBuf;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub fn fuzzy_filter(items: &[(PathBuf, usize)], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }

    let matcher = SkimMatcherV2::default();
    let query_lower = query.to_lowercase();

    let mut matches: Vec<(usize, i64)> = items
        .iter()
        .enumerate()
        .filter_map(|(idx, (path, _))| {
            // Only match files, not directories
            if path.is_dir() {
                // Include directories that have matching children
                return None;
            }

            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            matcher
                .fuzzy_match(&filename, &query_lower)
                .map(|score| (idx, score))
        })
        .collect();

    // Sort by score descending
    matches.sort_by(|a, b| b.1.cmp(&a.1));

    // Now include parent directories of matched files
    let matched_file_indices: Vec<usize> = matches.iter().map(|(idx, _)| *idx).collect();

    let mut result: Vec<usize> = Vec::new();
    let mut seen_parents: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for file_idx in matched_file_indices {
        let (file_path, _) = &items[file_idx];

        // Add all parent directories
        let mut current = file_path.parent();
        while let Some(parent) = current {
            if !seen_parents.contains(parent) {
                seen_parents.insert(parent.to_path_buf());
                // Find the index of this parent in items
                if let Some(parent_idx) = items.iter().position(|(p, _)| p == parent) {
                    if !result.contains(&parent_idx) {
                        result.push(parent_idx);
                    }
                }
            }
            current = parent.parent();
        }

        result.push(file_idx);
    }

    // Sort by original order to maintain tree structure
    result.sort();
    result
}
