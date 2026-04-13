use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::presentation::ui::app::{App, ConfigField};

pub fn render_config(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let header = Paragraph::new("Edit settings. Enter saves. Esc discards. Space toggles Sound.")
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
                &app.config().google.api_url,
            ),
            (
                ConfigField::GoogleApiKey,
                "API Key",
                &app.config().google.api_key,
            ),
            (
                ConfigField::GoogleModel,
                "Model",
                &app.config().google.model,
            ),
        ],
        app.config_field(),
    );

    render_provider_block(
        frame,
        chunks[2],
        "Groq",
        [
            (
                ConfigField::GroqApiUrl,
                "API URL",
                &app.config().groq.api_url,
            ),
            (
                ConfigField::GroqApiKey,
                "API Key",
                &app.config().groq.api_key,
            ),
            (ConfigField::GroqModel, "Model", &app.config().groq.model),
        ],
        app.config_field(),
    );

    render_game_settings_block(frame, chunks[3], app);

    let footer_text = app
        .status_message()
        .unwrap_or("Up/Down: move focus  Backspace: delete  Space: toggle Sound  Enter: save");
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[4]);
}

fn render_game_settings_block(frame: &mut Frame, area: Rect, app: &App) {
    let game = &app.config().game;
    let focused = app.config_field();

    let fields: [(ConfigField, &str, String); 4] = [
        (ConfigField::GameTimeout, "Timeout", game.timeout.clone()),
        (
            ConfigField::GameTextScale,
            "TextScale",
            game.text_scale.clone(),
        ),
        (ConfigField::GameFreq, "Freq", game.freq.clone()),
        (
            ConfigField::GameSoundEnabled,
            "Sound",
            if game.sound_enabled_value() {
                "enabled".to_string()
            } else {
                "disabled".to_string()
            },
        ),
    ];

    let mut lines = Vec::new();
    for (field, label, value) in &fields {
        let is_focused = *field == focused;
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
        lines.push(Line::from(vec![
            Span::styled(format!("{label:>9}: "), label_style),
            Span::styled(value.clone(), value_style),
        ]));
        lines.push(Line::from(""));
    }

    let block = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Game Settings ")
                .border_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(block, area);
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
            if value.is_empty() {
                String::new()
            } else {
                "********".to_string()
            }
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
