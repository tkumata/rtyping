use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::presentation::ui::app::{App, MenuItem};

use super::common::{centered_rect, render_help_overlay};

pub fn render_menu(frame: &mut Frame, app: &App) {
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
    let [_, title_area, menu_area, hint_area, _] = &*chunks else {
        return;
    };

    let title_text = logo_lines();

    let title = Paragraph::new(title_text)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(title, *title_area);

    let menu_lines = vec![
        menu_line(app, MenuItem::StartGame, "Start Game"),
        menu_line(app, MenuItem::PracticeMode, "Practice Mode"),
        menu_line(
            app,
            MenuItem::StartGameGoogle,
            "Start Game via Google AI Studio",
        ),
        menu_line(app, MenuItem::StartGameGroq, "Start Game via Groq"),
        menu_line(app, MenuItem::Config, "Config"),
    ];
    let menu = Paragraph::new(menu_lines)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(menu, *menu_area);

    let hint = Paragraph::new(vec![Line::from(vec![
        Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
        Span::raw(" to select, "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" to confirm, "),
        Span::styled("h", Style::default().fg(Color::Yellow)),
        Span::raw(" for help"),
    ])])
    .alignment(Alignment::Center);
    frame.render_widget(hint, *hint_area);

    if let Some(message) = app.status_message() {
        let message_area = centered_rect(70, 15, area);
        frame.render_widget(Clear, message_area);
        frame.render_widget(
            Paragraph::new(message)
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

    if app.is_help_visible() {
        render_help_overlay(frame, app.help_scroll());
    }
}

fn menu_line(app: &App, item: MenuItem, label: &str) -> Line<'static> {
    let selected = app.menu_selected() == item;
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

fn logo_lines() -> Vec<Line<'static>> {
    const LOGO_COLORS: [Color; 6] = [
        Color::Rgb(198, 229, 255),
        Color::Rgb(214, 236, 255),
        Color::Rgb(228, 242, 255),
        Color::Rgb(239, 247, 255),
        Color::Rgb(247, 251, 255),
        Color::Rgb(255, 255, 255),
    ];

    [
        " ____     _____            _             ",
        "|  _ \\   |_   _|   _ _ __ (_)_ __   __ _ ",
        "| |_) | _  | || | | | '_ \\| | '_ \\ / _` |",
        "|  _ < (_) | || |_| | |_) | | | | | (_| |",
        "|_| \\_\\    |_| \\__, | .__/|_|_| |_|\\__, |",
        "               |___/|_|            |___/ ",
    ]
    .into_iter()
    .zip(LOGO_COLORS)
    .map(|(line, color)| Line::from(Span::styled(line, Style::default().fg(color))))
    .collect()
}
