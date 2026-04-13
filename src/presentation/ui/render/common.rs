use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

const HELP_TEXT: &str = include_str!("../../../../docs/HELP.md");
const DECORATION_BLOCK: &str = "▗ ████████ ▖\n ▚█▙████▟█▞\n  ████████\n   ▛    ▜";

pub fn help_line_count() -> u16 {
    u16::try_from(HELP_TEXT.lines().count()).unwrap_or(u16::MAX)
}

pub fn render_help_overlay(frame: &mut Frame, scroll: u16) {
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

pub fn render_decoration_block(frame: &mut Frame, area: Rect) {
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

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
