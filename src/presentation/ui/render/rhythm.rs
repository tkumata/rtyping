use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::domain::rhythm::{RhythmJudgement, RhythmSession};
use crate::presentation::ui::app::App;

pub fn render_rhythm(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);
    let [header_area, lane_area, footer_area] = &*chunks else {
        return;
    };

    let header = Paragraph::new(format!(
        "Rhythm Mode  Speed: {} chars/sec  Judge: {}  Miss: {}  Hit+OK: {}",
        app.rhythm_speed(),
        judgement_label(app.rhythm_last_judgement()),
        app.rhythm_stats().map_or(0, |stats| stats.miss),
        app.rhythm_stats().map_or(0, |stats| stats.correct),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Rhythm ")
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .alignment(Alignment::Center);
    frame.render_widget(header, *header_area);

    let lane_width = lane_area.width.saturating_sub(2);
    let target_line = rhythm_target_line(app, lane_width);
    let marker_line = rhythm_marker_line(lane_width, app.rhythm_last_judgement());
    let combo_line = rhythm_combo_line(lane_width, app.rhythm_combo());
    let lane = Paragraph::new(vec![Line::from(""), target_line, marker_line, combo_line])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Flow ")
                .border_style(Style::default().fg(Color::Green)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(lane, *lane_area);

    let footer = Paragraph::new("Esc: return to menu")
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(footer, *footer_area);
}

fn judgement_label(judgement: Option<RhythmJudgement>) -> &'static str {
    judgement.map_or("-", RhythmJudgement::label)
}

fn rhythm_target_line(app: &App, width: u16) -> Line<'static> {
    let mut chars = vec![' '; usize::from(width)];
    for (column, ch) in app.rhythm_visible_chars(width) {
        if let Some(slot) = chars.get_mut(usize::from(column)) {
            *slot = ch;
        }
    }

    Line::from(Span::styled(
        chars.into_iter().collect::<String>(),
        Style::default().fg(Color::White),
    ))
}

fn rhythm_marker_line(width: u16, judgement: Option<RhythmJudgement>) -> Line<'static> {
    let mut chars = vec![' '; usize::from(width)];
    if let Ok(marker_column) = usize::try_from(RhythmSession::MARK_COLUMN)
        && let Some(slot) = chars.get_mut(marker_column)
    {
        *slot = '^';
        if let Some(judgement) = judgement {
            write_text(
                &mut chars,
                marker_column.saturating_add(2),
                judgement.label(),
            );
        }
    }

    Line::from(Span::styled(
        chars.into_iter().collect::<String>(),
        marker_style(judgement),
    ))
}

fn rhythm_combo_line(width: u16, combo: usize) -> Line<'static> {
    let mut chars = vec![' '; usize::from(width)];
    if combo >= 2
        && let Ok(marker_column) = usize::try_from(RhythmSession::MARK_COLUMN)
    {
        write_text(&mut chars, marker_column, &format!("{combo} Combo!!"));
    }

    Line::from(Span::styled(
        chars.into_iter().collect::<String>(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn write_text(chars: &mut [char], start: usize, text: &str) {
    for (offset, ch) in text.chars().enumerate() {
        if let Some(slot) = chars.get_mut(start.saturating_add(offset)) {
            *slot = ch;
        }
    }
}

fn marker_style(judgement: Option<RhythmJudgement>) -> Style {
    let color = match judgement {
        Some(RhythmJudgement::Hit) => Color::Green,
        Some(RhythmJudgement::Ok) | None => Color::Yellow,
        Some(RhythmJudgement::Miss) => Color::Red,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::{judgement_label, rhythm_combo_line, rhythm_marker_line};
    use crate::domain::rhythm::RhythmJudgement;

    #[test]
    fn marker_is_third_column_from_left_edge() {
        let line = rhythm_marker_line(8, None);
        assert_eq!(line.to_string(), "   ^    ");
    }

    #[test]
    fn marker_line_shows_judgement_near_marker() {
        let line = rhythm_marker_line(12, Some(RhythmJudgement::Hit));
        assert_eq!(line.to_string(), "   ^ Hit    ");
    }

    #[test]
    fn combo_line_is_blank_below_two_combo() {
        let line = rhythm_combo_line(12, 1);
        assert_eq!(line.to_string(), "            ");
    }

    #[test]
    fn combo_line_shows_combo_near_marker() {
        let line = rhythm_combo_line(16, 10);
        assert_eq!(line.to_string(), "   10 Combo!!   ");
    }

    #[test]
    fn judgement_label_formats_realtime_status() {
        assert_eq!(judgement_label(None), "-");
        assert_eq!(judgement_label(Some(RhythmJudgement::Hit)), "Hit");
        assert_eq!(judgement_label(Some(RhythmJudgement::Ok)), "OK");
        assert_eq!(judgement_label(Some(RhythmJudgement::Miss)), "Miss");
    }
}
