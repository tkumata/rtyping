use ratatui::text::Line;

use crate::usecase::history_stats::{HistoryStats, MistakeCount};

pub(super) fn history_summary_lines(stats: &HistoryStats) -> Vec<Line<'static>> {
    if stats.count == 0 {
        return vec![
            Line::from("History Stats"),
            Line::from(""),
            Line::from("No timed history yet"),
        ];
    }

    vec![
        Line::from("History Stats"),
        Line::from(""),
        Line::from(format!("Runs: {}", stats.count)),
        Line::from(format_optional("Best WPM", stats.best_wpm)),
        Line::from(format_optional("Avg WPM", stats.average_wpm)),
        Line::from(format_optional("Avg Accuracy", stats.average_accuracy)),
        Line::from(format!("Recent 10 WPM: {}", format_recent_wpm(stats))),
        Line::from(format!("Frequent misses: {}", format_mistakes(stats))),
    ]
}

fn format_optional(label: &str, value: Option<f64>) -> String {
    match value {
        Some(value) => format!("{label}: {value:.1}"),
        None => format!("{label}: -"),
    }
}

fn format_recent_wpm(stats: &HistoryStats) -> String {
    if stats.recent_wpm.is_empty() {
        return "-".into();
    }

    stats
        .recent_wpm
        .iter()
        .map(|wpm| format!("{wpm:.0}"))
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn format_mistakes(stats: &HistoryStats) -> String {
    if stats.frequent_mistakes.is_empty() {
        return "-".into();
    }

    stats
        .frequent_mistakes
        .iter()
        .map(format_mistake)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_mistake(mistake: &MistakeCount) -> String {
    format!("{}:{}", mistake.character, mistake.count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_summary_lines_show_empty_message() {
        let stats = HistoryStats {
            count: 0,
            best_wpm: None,
            average_wpm: None,
            average_accuracy: None,
            recent_wpm: Vec::new(),
            frequent_mistakes: Vec::new(),
        };

        let lines = history_summary_lines(&stats);

        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn history_summary_lines_format_recent_and_mistakes() {
        let stats = HistoryStats {
            count: 2,
            best_wpm: Some(30.0),
            average_wpm: Some(25.0),
            average_accuracy: Some(97.5),
            recent_wpm: vec![20.0, 30.0],
            frequent_mistakes: vec![MistakeCount {
                character: 'a',
                count: 2,
            }],
        };

        let lines = history_summary_lines(&stats);

        assert_eq!(lines.len(), 8);
    }
}
