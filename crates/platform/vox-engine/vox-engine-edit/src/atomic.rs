//! Atomic file replacement: temporary file, then rename.
//!
//! Write-path failures (a partial `write_all` or a failing `sync_all`)
//! are not deterministically injectable without a seam; the funnel
//! below removes the temporary file on every failing step, and the
//! rename failure test pins that the rename-path cleanup is real.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use vox_shared::{VoxError, VoxResult};

/// Monotonic counter making temporary names unique per process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write `content` to `path` atomically.
///
/// The bytes land in a temporary file in the target directory, are
/// flushed and synced, and only then does `rename` put them in place.
/// Any failure - writing the temporary file or the rename itself -
/// removes the temporary file and leaves the original file
/// byte-for-byte intact.
pub fn write_atomic(path: &Path, content: &str) -> VoxResult<()> {
    let parent = path.parent().ok_or_else(|| VoxError::Edit {
        detail: format!("{} has no parent directory", path.display()),
    })?;
    let temp = temp_path(parent, path);
    let result = (|| {
        let mut file = fs::File::create(&temp).map_err(VoxError::from)?;
        file.write_all(content.as_bytes()).map_err(VoxError::from)?;
        file.flush().map_err(VoxError::from)?;
        file.sync_all().map_err(VoxError::from)?;
        drop(file);
        fs::rename(&temp, path).map_err(VoxError::from)
    })();
    if result.is_err() {
        // The rename may have consumed the temporary file; a failing
        // removal is then expected and stays ignored.
        let _ = fs::remove_file(&temp);
    }
    result
}

/// Temporary sibling of `path` inside `parent`.
fn temp_path(parent: &Path, path: &Path) -> PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let stem = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "vox".to_string());
    parent.join(format!(".{stem}.vox-tmp-{}-{unique}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vox-edit-atomic-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn write_put_content_in_place() {
        let dir = temp_dir("write");
        let target = dir.join("file.txt");
        write_atomic(&target, "hello").expect("write");
        assert_eq!(fs::read_to_string(&target).expect("read"), "hello");
    }

    #[test]
    fn rename_failure_leaves_original_intact_and_no_temp_files() {
        let dir = temp_dir("rename-fail");
        // A directory cannot be replaced by rename(file, dir); the call
        // must fail while leaving the target untouched.
        let target = dir.join("occupied");
        fs::create_dir_all(&target).expect("create target dir");
        let error = write_atomic(&target, "payload");
        assert!(error.is_err());
        assert!(target.is_dir());
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("list dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("vox-tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files must be cleaned up");
    }

    #[test]
    fn overwrite_replaces_content() {
        let dir = temp_dir("overwrite");
        let target = dir.join("file.txt");
        fs::write(&target, "old").expect("seed");
        write_atomic(&target, "new").expect("overwrite");
        assert_eq!(fs::read_to_string(&target).expect("read"), "new");
    }

    #[test]
    fn create_failure_leaves_no_temporary_file() {
        let dir = temp_dir("create-fail");
        // A file where the parent directory should be makes the
        // temporary create fail deterministically (ENOTDIR).
        let blocked = dir.join("blocked");
        fs::write(&blocked, "not a directory").expect("block");
        let target = blocked.join("file.txt");
        assert!(write_atomic(&target, "payload").is_err());
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("list dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("vox-tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files must be cleaned up");
        assert_eq!(
            fs::read_to_string(&blocked).expect("read"),
            "not a directory"
        );
    }
}
