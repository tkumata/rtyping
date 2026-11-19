use std::fs;
use std::io;
use std::path::Path;

use crate::domain::history::{HistoryEntry, HistoryLoadReport};

use super::file_write::{atomic_write, lock, unique_sibling, write_target};

pub(super) fn load_history_from_path(history_path: &Path) -> io::Result<HistoryLoadReport> {
    let Some(content) = read_history(history_path)? else {
        return Ok(HistoryLoadReport {
            entries: Vec::new(),
            warnings: Vec::new(),
        });
    };

    match serde_json::from_slice(&content) {
        Ok(entries) => Ok(HistoryLoadReport {
            entries,
            warnings: Vec::new(),
        }),
        Err(err) => Ok(HistoryLoadReport {
            entries: Vec::new(),
            warnings: vec![format!("Failed to load history: {err}")],
        }),
    }
}

pub(super) fn append_history_to_path(
    entry: HistoryEntry,
    history_path: &Path,
) -> io::Result<Vec<HistoryEntry>> {
    let target = write_target(history_path)?;
    let history_path = target.as_path();
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let _lock = lock(history_path)?;
    let mut entries: Vec<HistoryEntry> = match read_history(history_path)? {
        None => Vec::new(),
        Some(content) => {
            if let Ok(entries) = serde_json::from_slice(&content) {
                entries
            } else {
                atomic_write(&unique_sibling(history_path, "corrupt"), &content)?;
                Vec::new()
            }
        }
    };
    entries.push(entry);

    let content = serde_json::to_vec_pretty(&entries)
        .map_err(|err| io::Error::other(format!("failed to serialize history: {err}")))?;
    atomic_write(history_path, &content)?;
    Ok(entries)
}

fn read_history(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used)]

    use super::*;
    use crate::domain::history::HistoryMode;

    fn entry(wpm: f64) -> HistoryEntry {
        HistoryEntry {
            wpm,
            accuracy: 95.0,
            miss_count: 1,
            elapsed_seconds: 30,
            generation_source: "Local".into(),
            mode: HistoryMode::Timed,
            missed_chars: vec!['a'],
        }
    }

    #[test]
    fn load_missing_history_returns_empty_report() {
        let dir = TempDir::new();
        let path = dir.0.join("missing.json");

        let report = load_history_from_path(&path).expect("missing history should load");

        assert_eq!(report.entries, [] as [HistoryEntry; 0]);
        assert_eq!(report.warnings, [] as [String; 0]);
    }

    #[test]
    fn save_and_load_history_round_trip() {
        let dir = TempDir::new();
        let path = dir.0.join("nested").join("history.json");

        append_history_to_path(entry(42.0), &path).expect("history should save");
        let report = load_history_from_path(&path).expect("history should load");

        assert_eq!(report.entries, vec![entry(42.0)]);
        assert_eq!(report.warnings, [] as [String; 0]);
    }

    #[test]
    fn load_broken_history_returns_warning_and_empty_entries() {
        let dir = TempDir::new();
        let path = dir.0.join("history.json");
        fs::write(&path, "{").expect("broken history should be written");

        let report = load_history_from_path(&path).expect("broken history should not fail hard");

        assert_eq!(report.entries, [] as [HistoryEntry; 0]);
        assert_eq!(report.warnings.len(), 1);
    }

    #[test]
    fn append_keeps_parallel_results_and_preserves_corrupt_history() {
        let dir = TempDir::new();
        let path = dir.0.join("history.json");
        fs::write(&path, [0xff]).expect("corrupt history should be written");
        std::thread::scope(|scope| {
            for wpm in [12.0, 24.0] {
                let path = &path;
                scope.spawn(move || append_history_to_path(entry(wpm), path).expect("append"));
            }
        });
        let report = load_history_from_path(&path).expect("history should load");
        assert_eq!(report.entries.len(), 2);
        assert!(report.entries.contains(&entry(12.0)));
        assert!(report.entries.contains(&entry(24.0)));
        let backup = fs::read_dir(&dir.0)
            .expect("read directory")
            .filter_map(Result::ok)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("history.json.corrupt-")
            })
            .expect("backup exists");
        assert_eq!(fs::read(backup.path()).expect("read backup"), [0xff]);
    }

    #[cfg(unix)]
    #[test]
    fn append_keeps_a_symlink_and_updates_its_target() {
        let dir = TempDir::new();
        let target = dir.0.join("shared.json");
        let link = dir.0.join("history.json");
        append_history_to_path(entry(12.0), &target).expect("initial append");
        std::os::unix::fs::symlink("shared.json", &link).expect("link history");

        append_history_to_path(entry(24.0), &link).expect("append through link");

        assert_eq!(
            fs::read_link(&link).expect("link should remain"),
            std::path::PathBuf::from("shared.json")
        );
        let report = load_history_from_path(&target).expect("load shared history");
        assert_eq!(report.entries, [entry(12.0), entry(24.0)]);
    }

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            use rand::RngExt;
            let suffix = rand::rng().random::<u64>();
            let path = std::env::temp_dir().join(format!("rtyping-history-test-{suffix:016x}"));
            fs::create_dir(&path).expect("temp dir should be created");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
