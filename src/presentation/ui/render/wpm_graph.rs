use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Sparkline},
};

pub fn render_wpm_graph(frame: &mut Frame, area: Rect, history: &[u64], title: &str) {
    if area.height < 3 || area.width < 8 {
        return;
    }

    let data = graph_data(history);

    frame.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .data(&data)
            .style(Style::default().fg(Color::LightGreen)),
        area,
    );
}

fn graph_data(history: &[u64]) -> Vec<u64> {
    if history.is_empty() {
        vec![0]
    } else {
        history.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::graph_data;

    #[test]
    fn graph_data_returns_zero_placeholder_for_empty_history() {
        assert_eq!(graph_data(&[]), vec![0]);
    }

    #[test]
    fn graph_data_reuses_existing_history_values() {
        assert_eq!(graph_data(&[1, 4, 9]), vec![1, 4, 9]);
    }
}
