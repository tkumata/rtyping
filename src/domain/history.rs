use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub wpm: f64,
    pub accuracy: f64,
    pub miss_count: usize,
    pub elapsed_seconds: i32,
    pub generation_source: String,
    pub mode: HistoryMode,
    pub missed_chars: Vec<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryMode {
    Timed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryLoadReport {
    pub entries: Vec<HistoryEntry>,
    pub warnings: Vec<String>,
}
