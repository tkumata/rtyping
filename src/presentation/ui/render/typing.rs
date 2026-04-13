use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::presentation::ui::app::App;

use super::common::render_decoration_block;
use super::wpm_graph;

pub fn render_typing(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let content_height = 17_u16;
    let vertical_padding = area.height.saturating_sub(content_height) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(3),
            Constraint::Min(11),
            Constraint::Length(3),
            Constraint::Length(vertical_padding),
        ])
        .split(area);

    render_header(frame, chunks[1], app);
    render_typing_area(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);
    render_decoration_block(frame, chunks[4]);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let (time_label, time_text, time_color) = if app.timeout() <= 0 {
        ("Elapsed: ", format!("{:03}", app.timer()), Color::Cyan)
    } else {
        let time_remaining = app.timeout().saturating_sub(app.timer());
        let color = if time_remaining <= 10 {
            Color::Red
        } else if time_remaining <= 30 {
            Color::Yellow
        } else {
            Color::Green
        };
        ("Time: ", format!("{time_remaining:03}"), color)
    };

    let countdown_widget = vec![Line::from(vec![
        Span::styled(time_label, Style::default().fg(Color::Gray)),
        Span::styled(
            time_text,
            Style::default().fg(time_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" s", Style::default().fg(Color::Gray)),
    ])];

    frame.render_widget(
        Paragraph::new(countdown_widget)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center),
        header_chunks[0],
    );

    let provider_label = match app.generation_source() {
        crate::usecase::generate_sentence::GenerationSource::Local => "LOCAL",
        crate::usecase::generate_sentence::GenerationSource::Google => "GOOGLE",
        crate::usecase::generate_sentence::GenerationSource::Groq => "GROQ",
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " R-Typing ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("[{provider_label}]"),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, header_chunks[1]);

    let wpm_text = vec![Line::from(vec![
        Span::styled("WPM: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:05.1}", app.current_wpm()),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ])];
    frame.render_widget(
        Paragraph::new(wpm_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center),
        header_chunks[2],
    );
}

fn render_typing_area(frame: &mut Frame, area: Rect, app: &App) {
    let [graph_area, text_area] = split_typing_area(area);
    wpm_graph::render_wpm_graph(frame, graph_area, app.wpm_history(), " WPM Trend ");

    let mut text_spans = Vec::new();
    let input_len = app.input_chars().len();

    for (i, target_char) in app.target_string().chars().enumerate() {
        match i.cmp(&input_len) {
            std::cmp::Ordering::Less => {
                let input_char = app.input_chars()[i];
                if input_char == target_char {
                    text_spans.push(Span::styled(
                        input_char.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                } else {
                    text_spans.push(Span::styled(
                        target_char.to_string(),
                        Style::default().fg(Color::White).bg(Color::Red),
                    ));
                }
            }
            std::cmp::Ordering::Equal => {
                text_spans.push(Span::styled(
                    target_char.to_string(),
                    Style::default().fg(Color::Yellow),
                ));
            }
            std::cmp::Ordering::Greater => {
                text_spans.push(Span::styled(
                    target_char.to_string(),
                    Style::default().fg(Color::White),
                ));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(text_spans),
            Line::from(""),
            Line::from(""),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Target Text ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false }),
        text_area,
    );

    if let Some(cursor_position) = typing_cursor_position(text_area, app) {
        frame.set_cursor_position(cursor_position);
    }
}

fn split_typing_area(area: Rect) -> [Rect; 2] {
    let graph_height = if area.height >= 10 { 4 } else { 0 };
    let constraints = if graph_height > 0 {
        [Constraint::Length(graph_height), Constraint::Min(7)]
    } else {
        [Constraint::Length(0), Constraint::Min(7)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    [chunks[0], chunks[1]]
}

fn typing_cursor_position(area: Rect, app: &App) -> Option<Position> {
    let content_x = area.x.checked_add(1)?;
    let content_y = area.y.checked_add(3)?;
    let content_width = area.width.checked_sub(2)?;
    let content_height = area.height.checked_sub(4)?;
    if content_width == 0 || content_height == 0 {
        return None;
    }

    let cursor_index = app
        .input_chars()
        .len()
        .saturating_add(1)
        .min(app.target_string().chars().count());
    let (cursor_row, cursor_col) =
        wrapped_cursor_offset(app.target_string(), cursor_index, content_width)?;
    let cursor_x = content_x.checked_add(cursor_col)?;
    let cursor_y = content_y.checked_add(cursor_row)?;
    let max_x = area.x.checked_add(area.width.checked_sub(2)?)?;
    let max_y = area.y.checked_add(area.height.checked_sub(2)?)?;

    if cursor_x > max_x || cursor_y > max_y {
        return None;
    }

    Some(Position::new(cursor_x, cursor_y))
}

#[derive(Debug, Clone, Copy)]
struct WrappedChar {
    character_index: usize,
    width: u16,
}

fn wrapped_cursor_offset(
    text: &str,
    cursor_index: usize,
    content_width: u16,
) -> Option<(u16, u16)> {
    if content_width == 0 {
        return None;
    }

    let cursor_character_index = cursor_index.saturating_sub(1);
    for (row, line) in wrapped_lines(text, content_width).iter().enumerate() {
        let mut col = 0_u16;
        for wrapped_char in line {
            col = col.checked_add(wrapped_char.width)?;
            if wrapped_char.character_index == cursor_character_index {
                return Some((u16::try_from(row).ok()?, col));
            }
        }
    }

    if text.is_empty() {
        return Some((0, 0));
    }

    None
}

fn wrapped_lines(text: &str, content_width: u16) -> Vec<Vec<WrappedChar>> {
    let mut lines = Vec::new();
    let mut pending_line = Vec::new();
    let mut pending_word = Vec::new();
    let mut pending_whitespace = std::collections::VecDeque::<WrappedChar>::new();
    let mut line_width = 0_u16;
    let mut word_width = 0_u16;
    let mut whitespace_width = 0_u16;
    let mut non_whitespace_previous = false;

    for (character_index, character) in text.chars().enumerate() {
        let symbol_width = character_width(character);
        if symbol_width == 0 || symbol_width > content_width {
            continue;
        }

        let is_whitespace = character.is_whitespace();
        let word_found = non_whitespace_previous && is_whitespace;
        let untrimmed_overflow =
            pending_line.is_empty() && word_width + whitespace_width + symbol_width > content_width;

        if word_found || untrimmed_overflow {
            pending_line.extend(pending_whitespace.drain(..));
            line_width += whitespace_width;
            pending_line.append(&mut pending_word);
            line_width += word_width;
            whitespace_width = 0;
            word_width = 0;
        }

        let line_full = line_width >= content_width;
        let pending_word_overflow =
            symbol_width > 0 && line_width + whitespace_width + word_width >= content_width;

        if line_full || pending_word_overflow {
            let mut remaining_width = content_width.saturating_sub(line_width);
            lines.push(std::mem::take(&mut pending_line));
            line_width = 0;

            while let Some(wrapped_char) = pending_whitespace.front() {
                if wrapped_char.width > remaining_width {
                    break;
                }

                whitespace_width -= wrapped_char.width;
                remaining_width -= wrapped_char.width;
                pending_whitespace.pop_front();
            }

            if is_whitespace && pending_whitespace.is_empty() {
                continue;
            }
        }

        let wrapped_char = WrappedChar {
            character_index,
            width: symbol_width,
        };
        if is_whitespace {
            whitespace_width += symbol_width;
            pending_whitespace.push_back(wrapped_char);
        } else {
            word_width += symbol_width;
            pending_word.push(wrapped_char);
        }

        non_whitespace_previous = !is_whitespace;
    }

    pending_line.extend(pending_whitespace);
    pending_line.append(&mut pending_word);
    if !pending_line.is_empty() {
        lines.push(pending_line);
    }
    if lines.is_empty() {
        lines.push(Vec::new());
    }

    lines
}

fn character_width(character: char) -> u16 {
    u16::from(!character.is_ascii_control())
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Types: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:03}", app.typed_count()),
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" chars", Style::default().fg(Color::Gray)),
        ]))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center),
        footer_chunks[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Misses: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:03}", app.incorrects()),
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center),
        footer_chunks[1],
    );
}

#[cfg(test)]
mod tests {
    use super::{split_typing_area, typing_cursor_position, wrapped_cursor_offset};
    use crate::domain::config::AppConfig;
    use crate::presentation::ui::app::App;
    use ratatui::layout::{Position, Rect};

    fn new_app_with_target(target: &str, input_len: usize) -> App {
        let mut app = App::new(AppConfig::default());
        app.prepare_new_game(target.to_string());
        for _ in 0..input_len {
            app.push_char('a');
        }
        app
    }

    #[test]
    fn split_typing_area_separates_graph_and_text_when_height_allows() {
        let [graph_area, text_area] = split_typing_area(Rect::new(0, 0, 80, 12));

        assert_eq!(graph_area.height, 4);
        assert_eq!(text_area.y, graph_area.y + graph_area.height);
        assert!(text_area.height >= 7);
    }

    #[test]
    fn split_typing_area_hides_graph_when_height_is_small() {
        let [graph_area, text_area] = split_typing_area(Rect::new(0, 0, 80, 8));

        assert_eq!(graph_area.height, 0);
        assert_eq!(text_area.height, 8);
    }

    #[test]
    fn typing_cursor_position_tracks_current_input_offset() {
        let app = new_app_with_target("abcd", 2);
        let cursor = typing_cursor_position(Rect::new(0, 0, 20, 10), &app);

        assert_eq!(cursor, Some(Position::new(4, 3)));
    }

    #[test]
    fn typing_cursor_position_wraps_to_next_line_when_needed() {
        let app = new_app_with_target("abcdef", 5);
        let cursor = typing_cursor_position(Rect::new(0, 0, 6, 10), &app);

        assert_eq!(cursor, Some(Position::new(3, 4)));
    }

    #[test]
    fn typing_cursor_position_matches_word_wrapping() {
        let app = new_app_with_target("abcd efgh ijkl", 7);
        let cursor = typing_cursor_position(Rect::new(0, 0, 8, 10), &app);

        assert_eq!(cursor, Some(Position::new(4, 4)));
    }

    #[test]
    fn wrapped_cursor_offset_keeps_words_with_their_wrapped_line() {
        assert_eq!(wrapped_cursor_offset("abcd efgh ijkl", 8, 6), Some((1, 3)));
    }

    #[test]
    fn typing_cursor_position_returns_none_when_text_area_is_too_small() {
        let app = new_app_with_target("abcd", 2);

        assert_eq!(typing_cursor_position(Rect::new(0, 0, 2, 10), &app), None);
        assert_eq!(typing_cursor_position(Rect::new(0, 0, 20, 4), &app), None);
    }

    #[test]
    fn typing_cursor_position_returns_none_when_wrap_overflows_visible_height() {
        let app = new_app_with_target("abcdef", 5);

        assert_eq!(typing_cursor_position(Rect::new(0, 0, 3, 5), &app), None);
    }
}
