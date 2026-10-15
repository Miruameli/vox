//! Project tree traversal shared by the search tools.

use std::path::{Path, PathBuf};

/// Directory names never entered during a walk: dependency caches
/// and build output dwarf the source tree the model asked about.
const SKIPPED_DIRS: [&str; 3] = [".git", "node_modules", "target"];

/// Every regular file under `root`, skipping build output and
/// dotfiles.
///
/// Order is unspecified; callers that publish paths to the model
/// sort first, because a stable answer is part of the contract.
pub fn files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                let skipped = SKIPPED_DIRS.contains(&name.as_str());
                if !skipped && !name.starts_with('.') {
                    pending.push(path);
                }
            } else if !name.starts_with('.') {
                found.push(path);
            }
        }
    }
    found
}
