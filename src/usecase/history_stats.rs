use std::collections::BTreeMap;

use crate::domain::history::HistoryEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryStats {
    pub count: usize,
    pub best_wpm: Option<f64>,
    pub average_wpm: Option<f64>,
    pub average_accuracy: Option<f64>,
    pub recent_wpm: Vec<f64>,
    pub frequent_mistakes: Vec<MistakeCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MistakeCount {
    pub character: char,
    pub count: usize,
}

pub fn summarize(entries: &[HistoryEntry]) -> HistoryStats {
    let count = entries.len();
    if entries.is_empty() {
        return HistoryStats {
            count,
            best_wpm: None,
            average_wpm: None,
            average_accuracy: None,
            recent_wpm: Vec::new(),
            frequent_mistakes: Vec::new(),
        };
    }

    let divisor = f64::from(u32::try_from(count).unwrap_or(u32::MAX));
    let best_wpm = entries.iter().map(|entry| entry.wpm).reduce(f64::max);
    let average_wpm = Some(entries.iter().map(|entry| entry.wpm).sum::<f64>() / divisor);
    let average_accuracy = Some(entries.iter().map(|entry| entry.accuracy).sum::<f64>() / divisor);
    let recent_wpm = recent_wpm(entries);
    let frequent_mistakes = frequent_mistakes(entries);

    HistoryStats {
        count,
        best_wpm,
        average_wpm,
        average_accuracy,
        recent_wpm,
        frequent_mistakes,
    }
}

fn recent_wpm(entries: &[HistoryEntry]) -> Vec<f64> {
    const RECENT_LIMIT: usize = 10;

    entries
        .iter()
        .skip(entries.len().saturating_sub(RECENT_LIMIT))
        .map(|entry| entry.wpm)
        .collect()
}

fn frequent_mistakes(entries: &[HistoryEntry]) -> Vec<MistakeCount> {
    const MISTAKE_LIMIT: usize = 5;

    let mut counts = BTreeMap::<char, usize>::new();
    for character in entries
        .iter()
        .flat_map(|entry| entry.missed_chars.iter().copied())
    {
        *counts.entry(character).or_default() += 1;
    }

    let mut mistakes = counts
        .into_iter()
        .map(|(character, count)| MistakeCount { character, count })
        .collect::<Vec<_>>();
    mistakes.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.character.cmp(&right.character))
    });
    mistakes.truncate(MISTAKE_LIMIT);
    mistakes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::history::HistoryMode;

    fn entry(wpm: f64, accuracy: f64, missed_chars: Vec<char>) -> HistoryEntry {
        HistoryEntry {
            wpm,
            accuracy,
            miss_count: missed_chars.len(),
            elapsed_seconds: 60,
            generation_source: "Local".into(),
            mode: HistoryMode::Timed,
            missed_chars,
        }
    }

    #[test]
    fn summarize_empty_history_returns_empty_stats() {
        let stats = summarize(&[]);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.best_wpm, None);
        assert!(stats.recent_wpm.is_empty());
        assert!(stats.frequent_mistakes.is_empty());
    }

    #[test]
    fn summarize_history_calculates_core_stats() {
        let entries = [
            entry(10.0, 90.0, vec!['a']),
            entry(20.0, 80.0, vec!['b', 'a']),
            entry(15.0, 100.0, vec!['a']),
        ];

        let stats = summarize(&entries);

        assert_eq!(stats.count, 3);
        assert_eq!(stats.best_wpm, Some(20.0));
        assert_eq!(stats.average_wpm, Some(15.0));
        assert_eq!(stats.average_accuracy, Some(90.0));
        assert_eq!(stats.recent_wpm, vec![10.0, 20.0, 15.0]);
        assert_eq!(
            stats.frequent_mistakes,
            vec![
                MistakeCount {
                    character: 'a',
                    count: 3
                },
                MistakeCount {
                    character: 'b',
                    count: 1
                }
            ]
        );
    }

    #[test]
    fn summarize_recent_wpm_keeps_latest_ten_in_saved_order() {
        let entries = (0..12)
            .map(|index| entry(f64::from(index), 100.0, Vec::new()))
            .collect::<Vec<_>>();

        let stats = summarize(&entries);

        assert_eq!(
            stats.recent_wpm,
            vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0]
        );
    }
}
