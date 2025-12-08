use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppState};
use crate::usecase::wpm;

pub fn render(frame: &mut Frame, app: &App) {
    match app.state {
        AppState::Intro => render_intro(frame),
        AppState::Typing => render_typing(frame, app),
        AppState::Result => render_result(frame, app),
    }
}

fn render_intro(frame: &mut Frame) {
    let area = frame.area();

    // アスキーアートの高さ(6行) + メッセージ(2行) + 空行(1行) + ボーダー(2行) = 11行
    let content_height = 11;
    let vertical_padding = area.height.saturating_sub(content_height + 3) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(content_height),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title_text = vec![
        Line::from(vec![Span::styled(
            " ____     _____            _             ",
            Style::default().fg(Color::Blue),
        )]),
        Line::from(vec![Span::styled(
            "|  _ \\   |_   _|   _ _ __ (_)_ __   __ _ ",
            Style::default().fg(Color::LightBlue),
        )]),
        Line::from(vec![Span::styled(
            "| |_) | _  | || | | | '_ \\| | '_ \\ / _` |",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            "|  _ < (_) | || |_| | |_) | | | | | (_| |",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(vec![Span::styled(
            "|_| \\_\\    |_| \\__, | .__/|_|_| |_|\\__, |",
            Style::default().fg(Color::LightGreen),
        )]),
        Line::from(vec![Span::styled(
            "               |___/|_|            |___/ ",
            Style::default().fg(Color::Green),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Let's begin typing!",
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![Span::styled(
            "Go for high WPM.",
            Style::default().fg(Color::White),
        )]),
    ];

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" R-Typing ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Cyan));

    let title_paragraph = Paragraph::new(title_text)
        .block(title_block)
        .alignment(Alignment::Center);

    frame.render_widget(title_paragraph, chunks[1]);

    let help_text = vec![Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "ENTER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to start", Style::default().fg(Color::Gray)),
    ])];

    let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);

    frame.render_widget(help_paragraph, chunks[2]);
}

fn render_typing(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // ヘッダー(3行) + タイピングエリア(最小5行) + フッター(3行) = 11行
    let content_height = 11_u16;
    let vertical_padding = area.height.saturating_sub(content_height) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(vertical_padding),
        ])
        .split(area);

    render_header(frame, chunks[1], app);
    render_typing_area(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)])
        .split(area);

    let time_remaining = app.timeout - app.timer;
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
            Style::default()
                .fg(time_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" s", Style::default().fg(Color::Gray)),
    ])];

    let timer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let timer_paragraph = Paragraph::new(timer_text)
        .block(timer_block)
        .alignment(Alignment::Center);

    frame.render_widget(timer_paragraph, header_chunks[0]);

    let mut title_spans = vec![Span::styled(
        " R-Typing ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];

    if app.sound_enabled {
        title_spans.push(Span::styled(" ♪", Style::default().fg(Color::Yellow)));
    }

    let title_text = vec![Line::from(title_spans)];

    let title_paragraph = Paragraph::new(title_text).alignment(Alignment::Center);

    frame.render_widget(title_paragraph, header_chunks[1]);

    let wpm_current = if app.timer > 0 {
        wpm::calc_wpm(app.typed_count(), app.timer, app.incorrects as i32)
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

    let wpm_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let wpm_paragraph = Paragraph::new(wpm_text)
        .block(wpm_block)
        .alignment(Alignment::Center);

    frame.render_widget(wpm_paragraph, header_chunks[2]);
}

fn render_typing_area(frame: &mut Frame, area: Rect, app: &App) {
    let mut text_spans = Vec::new();
    let input_len = app.inputs.len();

    for (i, target_char) in app.target_string.chars().enumerate() {
        if i < input_len {
            let input_char = app.inputs[i];
            if input_char == target_char {
                text_spans.push(Span::styled(
                    input_char.to_string(),
                    Style::default().fg(Color::Green),
                ));
            } else {
                // Show target char with red background for visibility
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

    let typing_text = vec![Line::from(text_spans)];

    let typing_block = Block::default()
        .borders(Borders::ALL)
        .title(" Target Text ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Cyan));

    let typing_paragraph = Paragraph::new(typing_text)
        .block(typing_block)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);

    frame.render_widget(typing_paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let types_text = vec![Line::from(vec![
        Span::styled("Types: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:03}", app.typed_count()),
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" chars", Style::default().fg(Color::Gray)),
    ])];

    let types_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let types_paragraph = Paragraph::new(types_text)
        .block(types_block)
        .alignment(Alignment::Center);

    frame.render_widget(types_paragraph, footer_chunks[0]);

    let misses_text = vec![Line::from(vec![
        Span::styled("Misses: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:03}", app.incorrects),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" chars", Style::default().fg(Color::Gray)),
    ])];

    let misses_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let misses_paragraph = Paragraph::new(misses_text)
        .block(misses_block)
        .alignment(Alignment::Center);

    frame.render_widget(misses_paragraph, footer_chunks[1]);
}

fn render_result(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // 結果表示エリア(10行) + ヘルプ(3行) = 13行
    let content_height = 13_u16;
    let vertical_padding = area.height.saturating_sub(content_height) / 2;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let wpm_value = wpm::calc_wpm(app.typed_count(), app.timer, app.incorrects as i32);

    let result_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🏁 Result",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Time     : ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:03}", app.timer),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(" sec", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Total Typing    : ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:03}", app.typed_count()),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(" chars", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Total Misses    : ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:03}", app.incorrects),
                Style::default().fg(Color::Red),
            ),
            Span::styled(" chars", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Words Per Minute: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:05.1}", wpm_value),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" wpm", Style::default().fg(Color::Gray)),
        ]),
    ];

    let result_block = Block::default()
        .borders(Borders::ALL)
        .title(" Results ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Green));

    let result_paragraph = Paragraph::new(result_text)
        .block(result_block)
        .alignment(Alignment::Center);

    frame.render_widget(result_paragraph, chunks[1]);

    let help_text = vec![Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "ENTER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to exit", Style::default().fg(Color::Gray)),
    ])];

    let help_paragraph = Paragraph::new(help_text).alignment(Alignment::Center);

    frame.render_widget(help_paragraph, chunks[2]);
}
