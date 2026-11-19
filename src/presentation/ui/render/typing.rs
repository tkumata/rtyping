use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::presentation::ui::app::App;

use super::common::render_decoration_block;
use super::wpm_graph;

const TYPING_CONTENT_HEIGHT: u16 = 18;
const TYPING_AREA_MIN_HEIGHT: u16 = 12;
const TARGET_TEXT_MIN_HEIGHT: u16 = 8;

pub fn render_typing(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let vertical_padding = area.height.saturating_sub(TYPING_CONTENT_HEIGHT) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(3),
            Constraint::Min(TYPING_AREA_MIN_HEIGHT),
            Constraint::Length(3),
            Constraint::Length(vertical_padding),
        ])
        .split(area);
    let [_, header_area, typing_area, footer_area, decoration_area] = &*chunks else {
        return;
    };

    render_header(frame, *header_area, app);
    render_typing_area(frame, *typing_area, app);
    render_footer(frame, *footer_area, app);
    render_decoration_block(frame, *decoration_area);
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
    let [countdown_area, title_area, wpm_area] = &*header_chunks else {
        return;
    };

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
            .block(Block::default())
            .alignment(Alignment::Center),
        *countdown_area,
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
    frame.render_widget(title, *title_area);

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
                Block::default(),
                // .borders(Borders::ALL)
                // .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center),
        *wpm_area,
    );
}

fn render_typing_area(frame: &mut Frame, area: Rect, app: &App) {
    let [graph_area, text_area] = split_typing_area(area);
    wpm_graph::render_wpm_graph(frame, graph_area, app.wpm_history(), " WPM Trend ");

    let content_width = text_area.width.saturating_sub(2);
    let (target_text_lines, current_line) = target_text_lines(app, content_width);
    let visible_lines = usize::from(text_area.height.saturating_sub(2));
    let scroll = current_line.saturating_add(1).saturating_sub(visible_lines);
    let scroll = u16::try_from(scroll).unwrap_or(u16::MAX);

    frame.render_widget(
        Paragraph::new(target_text_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Target Text ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .scroll((scroll, 0)),
        text_area,
    );
}

fn target_text_lines(app: &App, content_width: u16) -> (Vec<Line<'static>>, usize) {
    let target_chars = app.target_string().chars().collect::<Vec<_>>();
    // Generated text is normalized to ASCII before entering the game.
    let width = usize::from(content_width.max(1));
    let mut lines = vec![Line::from(""), Line::from("")];
    let mut current_line = 0;
    let mut offset = 0;
    let mut remaining = target_chars.as_slice();
    while !remaining.is_empty() {
        let mut line_len = remaining.len().min(width);
        if remaining.len() > width
            && let Some(prefix) = remaining.get(..width)
            && let Some(space) = prefix.iter().rposition(|ch| *ch == ' ')
            && space > 0
        {
            line_len = space + 1;
        }
        let (line, rest) = remaining.split_at(line_len);
        if (offset..offset + line_len).contains(&app.input_chars().len()) {
            current_line = lines.len();
        }
        lines.push(Line::from(
            line.iter()
                .enumerate()
                .map(|(index, ch)| target_char_span(app, offset + index, *ch))
                .collect::<Vec<_>>(),
        ));
        offset += line_len;
        remaining = rest;
    }
    lines.extend([Line::from(""), Line::from("")]);
    (lines, current_line)
}

fn target_char_span(app: &App, index: usize, target_char: char) -> Span<'static> {
    match index.cmp(&app.input_chars().len()) {
        std::cmp::Ordering::Less => {
            let Some(input_char) = app.input_chars().get(index).copied() else {
                return Span::raw("");
            };
            if input_char == target_char {
                Span::styled(input_char.to_string(), Style::default().fg(Color::Green))
            } else {
                Span::styled(
                    target_char.to_string(),
                    Style::default().fg(Color::White).bg(Color::Red),
                )
            }
        }
        std::cmp::Ordering::Equal => Span::styled(
            target_char.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        std::cmp::Ordering::Greater => {
            Span::styled(target_char.to_string(), Style::default().fg(Color::Gray))
        }
    }
}

fn split_typing_area(area: Rect) -> [Rect; 2] {
    let graph_height = if area.height >= TYPING_AREA_MIN_HEIGHT {
        4
    } else {
        0
    };
    let constraints = if graph_height > 0 {
        [
            Constraint::Length(graph_height),
            Constraint::Min(TARGET_TEXT_MIN_HEIGHT),
        ]
    } else {
        [
            Constraint::Length(0),
            Constraint::Min(TARGET_TEXT_MIN_HEIGHT),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    let [graph_area, text_area] = &*chunks else {
        return [area, area];
    };
    [*graph_area, *text_area]
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(status) = app.status_message() {
        frame.render_widget(Paragraph::new(status).wrap(Wrap { trim: true }), area);
        return;
    }
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let [types_area, misses_area] = &*footer_chunks else {
        return;
    };

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
        .block(Block::default())
        .alignment(Alignment::Center),
        *types_area,
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
        .block(Block::default())
        .alignment(Alignment::Center),
        *misses_area,
    );
}

#[cfg(test)]
mod tests {
    use super::{split_typing_area, target_char_span, target_text_lines};
    use crate::domain::config::AppConfig;
    use crate::presentation::ui::app::App;
    use ratatui::{
        layout::Rect,
        style::{Color, Modifier},
    };

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
        assert_eq!(text_area.height, 8);
    }

    #[test]
    fn split_typing_area_keeps_graph_hidden_until_text_padding_fits() {
        let [graph_area, text_area] = split_typing_area(Rect::new(0, 0, 80, 11));

        assert_eq!(graph_area.height, 0);
        assert_eq!(text_area.height, 11);
    }

    #[test]
    fn split_typing_area_hides_graph_when_height_is_small() {
        let [graph_area, text_area] = split_typing_area(Rect::new(0, 0, 80, 8));

        assert_eq!(graph_area.height, 0);
        assert_eq!(text_area.height, 8);
    }

    #[test]
    fn target_text_lines_keep_two_blank_lines_around_single_line_text() {
        let app = new_app_with_target("abcd", 0);
        let (lines, _) = target_text_lines(&app, 20);
        let span_counts = lines
            .iter()
            .map(|line| line.spans.len())
            .collect::<Vec<_>>();

        assert_eq!(span_counts, vec![0, 0, 4, 0, 0]);
    }

    #[test]
    fn target_text_lines_place_bottom_padding_after_wrapped_text() {
        let app = new_app_with_target("abcdef", 0);
        let (lines, _) = target_text_lines(&app, 3);
        let span_counts = lines
            .iter()
            .map(|line| line.spans.len())
            .collect::<Vec<_>>();

        assert_eq!(span_counts, vec![0, 0, 3, 3, 0, 0]);
    }

    #[test]
    fn long_target_keeps_the_current_character_visible() {
        use ratatui::{Terminal, backend::TestBackend};
        for width in [32, 80] {
            let Ok(mut terminal) = Terminal::new(TestBackend::new(width, 24));
            let target = "abcd ".repeat(200);
            let app = new_app_with_target(target.trim_end(), 500);
            let Ok(_) = terminal.draw(|frame| super::render_typing(frame, &app));
            assert!(
                terminal
                    .backend()
                    .buffer()
                    .content()
                    .iter()
                    .any(|cell| { cell.symbol() == "a" && cell.fg == Color::Yellow })
            );
        }
    }

    #[test]
    fn wrap_keeps_the_current_space_in_the_buffer() {
        let app = new_app_with_target("abc def", 3);
        let (lines, row) = target_text_lines(&app, 3);
        assert_eq!(row, 3);
        let active = lines.get(row).and_then(|line| line.spans.first());
        assert!(
            active.is_some_and(|span| span.content == " " && span.style.fg == Some(Color::Yellow))
        );
    }

    #[test]
    fn current_target_character_is_yellow_and_bold() {
        let app = new_app_with_target("abcd", 2);
        let span = target_char_span(&app, 2, 'c');

        assert_eq!(span.style.fg, Some(Color::Yellow));
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
        assert!(!span.style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn future_target_characters_are_gray() {
        let app = new_app_with_target("abcd", 2);
        let span = target_char_span(&app, 3, 'd');

        assert_eq!(span.style.fg, Some(Color::Gray));
    }

    #[test]
    fn correct_input_characters_are_green() {
        let app = new_app_with_target("abcd", 2);
        let span = target_char_span(&app, 0, 'a');

        assert_eq!(span.style.fg, Some(Color::Green));
    }

    #[test]
    fn incorrect_input_characters_use_white_on_red() {
        let mut app = App::new(AppConfig::default());
        app.prepare_new_game("abcd".to_string());
        app.push_char('x');
        let span = target_char_span(&app, 0, 'a');

        assert_eq!(span.style.fg, Some(Color::White));
        assert_eq!(span.style.bg, Some(Color::Red));
    }
}
