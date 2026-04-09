use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::presentation::ui::app::App;
use crate::usecase::wpm;

use super::common::render_decoration_block;

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

    let time_remaining = app.timeout() - app.timer();
    let time_color = if time_remaining <= 10 {
        Color::Red
    } else if time_remaining <= 30 {
        Color::Yellow
    } else {
        Color::Green
    };

    let timer_text = vec![Line::from(vec![
        Span::styled("Time: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:03}", time_remaining),
            Style::default().fg(time_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" s", Style::default().fg(Color::Gray)),
    ])];

    frame.render_widget(
        Paragraph::new(timer_text)
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

    let wpm_current = if app.timer() > 0 {
        wpm::calc_wpm(
            app.current_input_count(),
            app.timer(),
            app.incorrects() as i32,
        )
    } else {
        0.0
    };
    let wpm_text = vec![Line::from(vec![
        Span::styled("WPM: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:05.1}", wpm_current),
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
    let mut text_spans = Vec::new();
    let input_len = app.input_chars().len();

    for (i, target_char) in app.target_string().chars().enumerate() {
        if i < input_len {
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
        } else if i == input_len {
            text_spans.push(Span::styled(
                target_char.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            ));
        } else {
            text_spans.push(Span::styled(
                target_char.to_string(),
                Style::default().fg(Color::White),
            ));
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
        area,
    );
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
                format!("{:03}", app.current_input_count()),
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
