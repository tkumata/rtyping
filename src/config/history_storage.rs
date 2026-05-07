use std::fs;
use std::io;
use std::path::Path;

use crate::domain::history::{HistoryEntry, HistoryLoadReport};

pub(super) fn load_history_from_path(history_path: &Path) -> io::Result<HistoryLoadReport> {
    if !history_path.exists() {
        return Ok(HistoryLoadReport {
            entries: Vec::new(),
            warnings: Vec::new(),
        });
    }

    let content = fs::read_to_string(history_path)?;
    match serde_json::from_str::<Vec<HistoryEntry>>(&content) {
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

pub(super) fn save_history_to_path(
    entries: &[HistoryEntry],
    history_path: &Path,
) -> io::Result<()> {
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(entries)
        .map_err(|err| io::Error::other(format!("failed to serialize history: {err}")))?;
    fs::write(history_path, content)
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
        let dir = tempfile_dir();
        let path = dir.join("missing.json");

        let report = load_history_from_path(&path).expect("missing history should load");

        assert!(report.entries.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn save_and_load_history_round_trip() {
        let dir = tempfile_dir();
        let path = dir.join("nested").join("history.json");

        save_history_to_path(&[entry(42.0)], &path).expect("history should save");
        let report = load_history_from_path(&path).expect("history should load");

        assert_eq!(report.entries, vec![entry(42.0)]);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn load_broken_history_returns_warning_and_empty_entries() {
        let dir = tempfile_dir();
        let path = dir.join("history.json");
        fs::write(&path, "{").expect("broken history should be written");

        let report = load_history_from_path(&path).expect("broken history should not fail hard");

        assert!(report.entries.is_empty());
        assert_eq!(report.warnings.len(), 1);
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "rtyping-history-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }
}
