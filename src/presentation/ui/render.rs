use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppState};
use crate::usecase::wpm;

const HELP_TEXT: &str = include_str!("../../../docs/HELP.md");

/// 右下に表示するUnicodeブロック要素
const DECORATION_BLOCK: &str = "▗ ████████ ▖\n ▚█▙████▟█▞\n  ████████\n   ▛    ▜";

/// ヘルプテキストの行数を返す
pub fn help_line_count() -> u16 {
    HELP_TEXT.lines().count() as u16
}

pub fn render(frame: &mut Frame, app: &App) {
    match app.state {
        AppState::Intro => render_intro(frame, app),
        AppState::Typing => render_typing(frame, app),
        AppState::Result => render_result(frame, app),
    }
}

fn render_intro(frame: &mut Frame, app: &App) {
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

    let help_hint_text = vec![Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::Gray)),
        Span::styled(
            "ENTER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to start, ", Style::default().fg(Color::Gray)),
        Span::styled(
            "h",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" for help", Style::default().fg(Color::Gray)),
    ])];

    let help_hint_paragraph = Paragraph::new(help_hint_text).alignment(Alignment::Center);

    frame.render_widget(help_hint_paragraph, chunks[2]);

    // ヘルプオーバーレイの描画
    if app.show_help {
        render_help_overlay(frame, app.help_scroll);
    }
}

fn render_typing(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // ヘッダー(3行) + タイピングエリア(最小11行: 上下3行パディング+ボーダー2行+内容3行) + フッター(3行) = 17行
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

    // 右下余白にブロック要素を表示
    render_decoration_block(frame, chunks[4]);
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

    // 上下に3行の空行を追加してテキストを中央に配置
    let typing_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(text_spans),
        Line::from(""),
        Line::from(""),
        Line::from(""),
    ];

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

fn render_help_overlay(frame: &mut Frame, scroll: u16) {
    let area = frame.area();

    // ヘルプブロックのサイズを計算（画面の80%幅、70%高さ）
    let popup_width = (area.width as f32 * 0.8) as u16;
    let popup_height = (area.height as f32 * 0.7) as u16;

    // 中央に配置
    let popup_area = centered_rect(popup_width, popup_height, area);

    // 背景をクリア
    frame.render_widget(Clear, popup_area);

    // ヘルプテキストをパースしてスタイル付きで表示
    let help_lines: Vec<Line> = HELP_TEXT
        .lines()
        .map(|line| {
            if line.starts_with("# ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("## ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("### ") {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("```") {
                Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)))
            } else if line.starts_with("- **") {
                Line::from(Span::styled(line, Style::default().fg(Color::White)))
            } else if line.starts_with("-") || line.starts_with("  -") {
                Line::from(Span::styled(line, Style::default().fg(Color::Gray)))
            } else {
                Line::from(Span::styled(line, Style::default().fg(Color::White)))
            }
        })
        .collect();

    let help_block = Block::default()
        .borders(Borders::ALL)
        .title(" Help (h: close, ↑↓: scroll) ")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Yellow));

    let help_paragraph = Paragraph::new(help_lines)
        .block(help_block)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left)
        .scroll((scroll, 0));

    frame.render_widget(help_paragraph, popup_area);
}

/// 中央に配置された矩形を計算するヘルパー関数
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// 右下余白にデコレーションブロックを表示
fn render_decoration_block(frame: &mut Frame, area: Rect) {
    // ブロックの高さは4行
    let block_height = 4_u16;
    // ブロックの幅は13文字（最長行の"▖ ████████ ▖"が13文字）
    let block_width = 13_u16;

    // 余白の高さと幅が十分にある場合のみ表示
    if area.height >= block_height && area.width >= block_width {
        // 右下に配置するための位置を計算
        let x = area.x + area.width.saturating_sub(block_width);
        let y = area.y + area.height.saturating_sub(block_height);

        let block_area = Rect::new(x, y, block_width, block_height);

        let block_lines: Vec<Line> = DECORATION_BLOCK
            .lines()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::DarkGray))))
            .collect();

        let block_paragraph = Paragraph::new(block_lines)
            .alignment(Alignment::Left);

        frame.render_widget(block_paragraph, block_area);
    }
}
