use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
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
    let [
        header_area,
        google_area,
        groq_area,
        game_area,
        footer_area,
        _,
    ] = &*chunks
    else {
        return;
    };

    let header = Paragraph::new("Edit settings. Enter saves. Esc discards. Space toggles Sound.")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Config ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(header, *header_area);

    let google_cursor_position = render_provider_block(
        frame,
        *google_area,
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

    let groq_cursor_position = render_provider_block(
        frame,
        *groq_area,
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

    let game_cursor_position = render_game_settings_block(frame, *game_area, app);
    let cursor_position = google_cursor_position
        .or(groq_cursor_position)
        .or(game_cursor_position);

    let footer_text = app
        .status_message()
        .unwrap_or("Up/Down: move focus  Backspace: delete  Space: toggle Sound  Enter: save");
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(footer, *footer_area);

    if let Some(cursor_position) = cursor_position {
        frame.set_cursor_position(cursor_position);
    }
}

fn render_game_settings_block(frame: &mut Frame, area: Rect, app: &App) -> Option<Position> {
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

    config_cursor_position(area, focused, &fields, 11)
}

fn render_provider_block(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    fields: [(ConfigField, &str, &String); 3],
    focused: ConfigField,
) -> Option<Position> {
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

    config_cursor_position(area, focused, &fields, 10)
}

fn config_cursor_position<V: AsRef<str>>(
    area: Rect,
    focused: ConfigField,
    fields: &[(ConfigField, &str, V)],
    label_width: u16,
) -> Option<Position> {
    let field_index = fields.iter().position(|(field, _, _)| *field == focused)?;
    let value = fields.get(field_index)?.2.as_ref();
    let row_offset = u16::try_from(field_index).ok()?.saturating_mul(2);
    let cursor_x = area
        .x
        .checked_add(1)?
        .checked_add(label_width)?
        .checked_add(u16::try_from(value.chars().count()).ok()?)?;
    let cursor_y = area.y.checked_add(1)?.checked_add(row_offset)?;

    Some(Position::new(cursor_x, cursor_y))
}

#[cfg(test)]
mod tests {
    use super::{ConfigField, config_cursor_position};
    use ratatui::layout::{Position, Rect};

    #[test]
    fn config_cursor_position_points_to_provider_value_end() {
        let fields = [
            (
                ConfigField::GoogleApiUrl,
                "API URL",
                String::from("https://"),
            ),
            (ConfigField::GoogleApiKey, "API Key", String::from("secret")),
            (ConfigField::GoogleModel, "Model", String::from("gemini")),
        ];

        let cursor = config_cursor_position(
            Rect::new(2, 4, 40, 10),
            ConfigField::GoogleApiKey,
            &fields,
            10,
        );

        assert_eq!(cursor, Some(Position::new(19, 7)));
    }

    #[test]
    fn config_cursor_position_points_to_game_value_end() {
        let fields = [
            (ConfigField::GameTimeout, "Timeout", String::from("60")),
            (ConfigField::GameTextScale, "TextScale", String::from("60")),
            (ConfigField::GameFreq, "Freq", String::from("80.0")),
            (
                ConfigField::GameSoundEnabled,
                "Sound",
                String::from("enabled"),
            ),
        ];

        let cursor = config_cursor_position(
            Rect::new(0, 0, 40, 12),
            ConfigField::GameSoundEnabled,
            &fields,
            11,
        );

        assert_eq!(cursor, Some(Position::new(19, 7)));
    }
}
