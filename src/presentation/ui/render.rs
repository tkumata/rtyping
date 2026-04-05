use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use super::app::{App, AppState, ConfigField, MenuItem};
use crate::usecase::wpm;

const HELP_TEXT: &str = include_str!("../../../docs/HELP.md");
const DECORATION_BLOCK: &str = "▗ ████████ ▖\n ▚█▙████▟█▞\n  ████████\n   ▛    ▜";

pub fn help_line_count() -> u16 {
    HELP_TEXT.lines().count() as u16
}

pub fn render(frame: &mut Frame, app: &App) {
    match app.state {
        AppState::Menu => render_menu(frame, app),
        AppState::Config => render_config(frame, app),
        AppState::Loading => render_loading(frame, app),
        AppState::Typing => render_typing(frame, app),
        AppState::Result => render_result(frame, app),
    }
}

fn render_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(11),
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title_text = vec![
        Line::from(" ____     _____            _             "),
        Line::from("|  _ \\   |_   _|   _ _ __ (_)_ __   __ _ "),
        Line::from("| |_) | _  | || | | | '_ \\| | '_ \\ / _` |"),
        Line::from("|  _ < (_) | || |_| | |_) | | | | | (_| |"),
        Line::from("|_| \\_\\    |_| \\__, | .__/|_|_| |_|\\__, |"),
        Line::from("               |___/|_|            |___/ "),
    ];

    let title = Paragraph::new(title_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" R-Typing ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    let menu_lines = vec![
        menu_line(app, MenuItem::StartGame, "Start Game"),
        Line::from(""),
        menu_line(app, MenuItem::Config, "Config"),
    ];
    let menu = Paragraph::new(menu_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Menu ")
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(menu, chunks[2]);

    let hint = Paragraph::new(vec![Line::from(vec![
        Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
        Span::raw(" to select, "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" to confirm, "),
        Span::styled("h", Style::default().fg(Color::Yellow)),
        Span::raw(" for help"),
    ])])
    .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);

    if let Some(message) = &app.status_message {
        let message_area = centered_rect(70, 15, area);
        frame.render_widget(Clear, message_area);
        frame.render_widget(
            Paragraph::new(message.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Status ")
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .wrap(Wrap { trim: true }),
            message_area,
        );
    }

    if app.show_help {
        render_help_overlay(frame, app.help_scroll);
    }
}

fn menu_line(app: &App, item: MenuItem, label: &str) -> Line<'static> {
    let selected = app.menu_selected == item;
    let pointer = if selected { ">" } else { " " };
    let style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    Line::from(vec![
        Span::styled(format!("{pointer} "), style),
        Span::styled(label.to_string(), style),
    ])
}

fn render_config(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let header = Paragraph::new("Edit provider settings. Enter saves. Esc returns to menu.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Config ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    render_provider_block(
        frame,
        chunks[1],
        "Google AI Studio",
        [
            (
                ConfigField::GoogleApiUrl,
                "API URL",
                &app.config.google.api_url,
            ),
            (
                ConfigField::GoogleApiKey,
                "API Key",
                &app.config.google.api_key,
            ),
            (ConfigField::GoogleModel, "Model", &app.config.google.model),
        ],
        app.config_field,
    );

    render_provider_block(
        frame,
        chunks[2],
        "Groq",
        [
            (ConfigField::GroqApiUrl, "API URL", &app.config.groq.api_url),
            (ConfigField::GroqApiKey, "API Key", &app.config.groq.api_key),
            (ConfigField::GroqModel, "Model", &app.config.groq.model),
        ],
        app.config_field,
    );

    let footer_text = if let Some(message) = &app.status_message {
        message.as_str()
    } else {
        "Up/Down move focus. Backspace deletes. API key is encrypted on save."
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[3]);
}

fn render_provider_block(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    fields: [(ConfigField, &str, &String); 3],
    focused: ConfigField,
) {
    let mut lines = Vec::new();
    for (field, label, value) in fields {
        let is_focused = field == focused;
        let label_style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let value_style = if is_focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };
        let display_value = if matches!(field, ConfigField::GoogleApiKey | ConfigField::GroqApiKey)
        {
            "*".repeat(value.chars().count())
        } else {
            value.clone()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{label:>8}: "), label_style),
            Span::styled(display_value, value_style),
        ]));
        lines.push(Line::from(""));
    }

    let block = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {title} "))
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(block, area);
}

fn render_loading(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);
    let provider = match app.generation_source {
        crate::usecase::generate_sentence::GenerationSource::Local => "Local",
        crate::usecase::generate_sentence::GenerationSource::Google => "Google AI Studio",
        crate::usecase::generate_sentence::GenerationSource::Groq => "Groq",
    };
    let text = vec![
        Line::from(format!("Generating text with {provider}")),
        Line::from(""),
        Line::from("Please wait..."),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Loading ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center),
        area,
    );
}

fn render_typing(frame: &mut Frame, app: &App) {
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

    let provider_label = match app.generation_source {
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
                format!("{:03}", app.incorrects),
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

fn render_result(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 35, frame.area());
    let elapsed = app.timer.max(1);
    let score = wpm::calc_wpm(app.typed_count(), elapsed, app.incorrects as i32);
    let lines = vec![
        Line::from("Game Finished"),
        Line::from(""),
        Line::from(format!("Typed: {}", app.typed_count())),
        Line::from(format!("Misses: {}", app.incorrects)),
        Line::from(format!("Time: {} sec", elapsed)),
        Line::from(format!("WPM: {:.1}", score)),
        Line::from(""),
        Line::from("Press Enter to quit"),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Result ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Center),
        area,
    );
}

fn render_help_overlay(frame: &mut Frame, scroll: u16) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(HELP_TEXT)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_decoration_block(frame: &mut Frame, area: Rect) {
    let block_height = 4_u16;
    let block_width = 13_u16;

    if area.height >= block_height && area.width >= block_width {
        let x = area.x + area.width.saturating_sub(block_width);
        let y = area.y + area.height.saturating_sub(block_height);
        let block_area = Rect::new(x, y, block_width, block_height);
        let block_lines: Vec<Line> = DECORATION_BLOCK
            .lines()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::DarkGray))))
            .collect();

        frame.render_widget(Paragraph::new(block_lines), block_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
