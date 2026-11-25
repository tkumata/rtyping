use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::presentation::ui::app::{App, ConfigField};

const GAME_LABEL_WIDTH: u16 = 13;
const PROVIDER_LABEL_WIDTH: u16 = 10;

pub fn render_config(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(11),
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

    let header = Paragraph::new(vec![
        Line::styled("Config", Style::default().fg(Color::Cyan)),
        Line::from("Edit settings. Enter saves. Esc discards. Space toggles Sound."),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    frame.render_widget(header, *header_area);

    let google_cursor_position = render_provider_block(
        frame,
        *google_area,
        "Google AI Studio",
        [
            (
                ConfigField::GoogleApiUrl,
                "API URL",
                &app.config_draft().google.api_url,
            ),
            (
                ConfigField::GoogleApiKey,
                "API Key",
                &app.config_draft().google.api_key,
            ),
            (
                ConfigField::GoogleModel,
                "Model",
                &app.config_draft().google.model,
            ),
        ],
        app.config_field(),
        app.config_cursor_index(),
    );

    let groq_cursor_position = render_provider_block(
        frame,
        *groq_area,
        "Groq",
        [
            (
                ConfigField::GroqApiUrl,
                "API URL",
                &app.config_draft().groq.api_url,
            ),
            (
                ConfigField::GroqApiKey,
                "API Key",
                &app.config_draft().groq.api_key,
            ),
            (
                ConfigField::GroqModel,
                "Model",
                &app.config_draft().groq.model,
            ),
        ],
        app.config_field(),
        app.config_cursor_index(),
    );

    let game_cursor_position = render_game_settings_block(frame, *game_area, app);
    let cursor_position = google_cursor_position
        .or(groq_cursor_position)
        .or(game_cursor_position);

    let footer_text = app.status_message().unwrap_or(
        "Up/Down: focus  Left/Right: cursor  Backspace: delete before cursor  Enter: save",
    );
    let footer = Paragraph::new(footer_text).wrap(Wrap { trim: true });
    frame.render_widget(footer, *footer_area);

    if let Some(cursor_position) = cursor_position {
        frame.set_cursor_position(cursor_position);
    }
}

fn render_game_settings_block(frame: &mut Frame, area: Rect, app: &App) -> Option<Position> {
    let game = &app.config_draft().game;
    let focused = app.config_field();

    let fields: [(ConfigField, &str, String); 5] = [
        (ConfigField::GameTimeout, "Timeout", game.timeout.clone()),
        (
            ConfigField::GameTextScale,
            "TextScale",
            game.text_scale.clone(),
        ),
        (
            ConfigField::GameRhythmSpeed,
            "RhythmSpeed",
            game.rhythm_speed.clone(),
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
        if !lines.is_empty() {
            lines.push(Line::default());
        }
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
        let (value, _) = visible_config_value(
            value,
            area.width.saturating_sub(GAME_LABEL_WIDTH + 2),
            if is_focused {
                app.config_cursor_index()
            } else {
                0
            },
        );
        lines.push(Line::from(vec![
            Span::styled(format!("{label:>11}: "), label_style),
            Span::styled(value.to_owned(), value_style),
        ]));
    }

    let block = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Game Settings ")
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(block, area);

    config_cursor_position(
        area,
        focused,
        &fields,
        GAME_LABEL_WIDTH,
        app.config_cursor_index(),
    )
}

fn render_provider_block(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    fields: [(ConfigField, &str, &String); 3],
    focused: ConfigField,
    cursor_index: usize,
) -> Option<Position> {
    let mut lines = Vec::new();
    for (field, label, value) in fields {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
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
        let display_value = masked_config_value(field, value);
        let (display_value, _) = visible_config_value(
            &display_value,
            area.width.saturating_sub(PROVIDER_LABEL_WIDTH + 2),
            if is_focused { cursor_index } else { 0 },
        );

        lines.push(Line::from(vec![
            Span::styled(format!("{label:>8}: "), label_style),
            Span::styled(display_value.to_owned(), value_style),
        ]));
    }

    let block = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {title} "))
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(block, area);

    config_cursor_position(area, focused, &fields, PROVIDER_LABEL_WIDTH, cursor_index)
}

fn config_cursor_position<V: AsRef<str>>(
    area: Rect,
    focused: ConfigField,
    fields: &[(ConfigField, &str, V)],
    label_width: u16,
    cursor_index: usize,
) -> Option<Position> {
    if !focused.accepts_text() {
        return None;
    }

    let field_index = fields.iter().position(|(field, _, _)| *field == focused)?;
    let value = masked_config_value(focused, fields.get(field_index)?.2.as_ref());
    let value_width = area.width.checked_sub(label_width + 2)?;
    if value_width == 0 {
        return None;
    }
    let (_, cursor_column) = visible_config_value(&value, value_width, cursor_index);
    let row_offset = u16::try_from(field_index).ok()?.saturating_mul(2);
    if row_offset + 2 >= area.height {
        return None;
    }
    let cursor_x = area
        .x
        .checked_add(1)?
        .checked_add(label_width)?
        .checked_add(u16::try_from(cursor_column).ok()?)?;
    let cursor_y = area.y.checked_add(1)?.checked_add(row_offset)?;

    Some(Position::new(cursor_x, cursor_y))
}

fn masked_config_value(field: ConfigField, value: &str) -> std::borrow::Cow<'_, str> {
    if matches!(field, ConfigField::GoogleApiKey | ConfigField::GroqApiKey) {
        "*".repeat(value.chars().count()).into()
    } else {
        value.into()
    }
}

fn visible_config_value(value: &str, width: u16, cursor_index: usize) -> (&str, usize) {
    let cursor = value
        .char_indices()
        .nth(cursor_index)
        .map_or(value.len(), |(index, _)| index);
    let value_span = Span::raw(value);
    let preceding_glyphs = || {
        value_span
            .styled_graphemes(Style::default())
            .map(|glyph| {
                let end = glyph.symbol.as_ptr().addr() - value.as_ptr().addr() + glyph.symbol.len();
                (end, Span::raw(glyph.symbol).width())
            })
            .take_while(move |&(end, _)| end <= cursor)
    };
    let mut column: usize = preceding_glyphs().map(|(_, columns)| columns).sum();
    let mut start = 0;
    let last_column = usize::from(width.saturating_sub(1));
    for (end, columns) in preceding_glyphs() {
        if column <= last_column {
            break;
        }
        column -= columns;
        start = end;
    }
    (value.split_at(start).1, column)
}

#[cfg(test)]
mod tests {
    use super::{ConfigField, PROVIDER_LABEL_WIDTH, config_cursor_position, visible_config_value};
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
            3,
        );

        assert_eq!(cursor, Some(Position::new(16, 7)));
    }

    #[test]
    fn config_cursor_position_points_to_game_value_cursor() {
        let fields = [
            (ConfigField::GameTimeout, "Timeout", String::from("60")),
            (ConfigField::GameTextScale, "TextScale", String::from("60")),
            (ConfigField::GameFreq, "Freq", String::from("80.0")),
            (
                ConfigField::GameRhythmSpeed,
                "RhythmSpeed",
                String::from("2"),
            ),
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
            13,
            1,
        );

        assert_eq!(cursor, None);
    }

    #[test]
    fn config_cursor_position_clamps_to_value_end() {
        let fields = [(ConfigField::GoogleApiKey, "API Key", String::from("secret"))];

        let cursor = config_cursor_position(
            Rect::new(2, 4, 40, 10),
            ConfigField::GoogleApiKey,
            &fields,
            10,
            99,
        );

        assert_eq!(cursor, Some(Position::new(19, 5)));
    }

    #[test]
    fn long_config_value_scrolls_to_keep_the_cursor_inside_the_field() {
        assert_eq!(visible_config_value("12345678", 4, 6), ("45678", 3));
        let fields = [(ConfigField::GoogleApiKey, "API Key", "12345678")];
        assert_eq!(
            config_cursor_position(
                Rect::new(0, 0, 16, 7),
                ConfigField::GoogleApiKey,
                &fields,
                PROVIDER_LABEL_WIDTH,
                6,
            ),
            Some(Position::new(14, 1)),
        );
    }

    #[test]
    fn config_cursor_uses_display_width_without_splitting_a_glyph() {
        assert_eq!(visible_config_value("👩‍💻Z", 4, 3), ("👩‍💻Z", 2));
        assert_eq!(visible_config_value("👩‍💻Z", 2, 3), ("Z", 0));
        assert_eq!(visible_config_value("👩‍💻Z", 2, 1), ("👩‍💻Z", 0));
    }
}
