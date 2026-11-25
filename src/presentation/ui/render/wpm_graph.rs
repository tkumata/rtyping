use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::{
        Block,
        canvas::{Canvas, Line as CanvasLine, Points},
    },
};

pub fn render_wpm_graph(frame: &mut Frame, area: Rect, history: &[u64], block: Block<'_>) {
    if area.height < 3 || area.width < 8 {
        return;
    }

    frame.render_widget(
        Canvas::default()
            .marker(Marker::Braille)
            .block(block)
            .background_color(Color::Reset)
            .x_bounds(graph_x_bounds(history))
            .y_bounds(graph_y_bounds(history))
            .paint(|ctx| paint_graph(ctx, history)),
        area,
    );
}

fn paint_graph(ctx: &mut ratatui::widgets::canvas::Context<'_>, history: &[u64]) {
    if history.is_empty() {
        ctx.draw(&Points::new(&[(0.0, 0.0)], Color::LightGreen));
        return;
    }

    let max_value = history.iter().copied().max().unwrap_or(0);
    let points: Vec<_> = history
        .iter()
        .enumerate()
        .map(|(index, value)| (to_f64_usize(index), to_f64_u64(*value)))
        .collect();

    for (index, window) in points.windows(2).enumerate() {
        let [start, end] = window else {
            continue;
        };
        let [start, end] = [*start, *end];
        let color = if index == 0 {
            Color::LightGreen
        } else if should_highlight_pair(start.1, end.1, max_value) {
            Color::Rgb(255, 165, 0)
        } else {
            Color::LightGreen
        };
        ctx.draw(&CanvasLine::new(start.0, start.1, end.0, end.1, color));
    }

    ctx.draw(&Points::new(&points, Color::LightGreen));
    let highlight_points: Vec<(f64, f64)> = points
        .iter()
        .enumerate()
        .zip(history.iter().copied())
        .filter_map(
            |((index, &(x, y)), value)| match point_color(index, value, max_value) {
                Color::Rgb(255, 165, 0) => Some((x, y)),
                _ => None,
            },
        )
        .collect();
    if !highlight_points.is_empty() {
        ctx.draw(&Points::new(&highlight_points, Color::Rgb(255, 165, 0)));
    }
}

fn graph_x_bounds(history: &[u64]) -> [f64; 2] {
    match history.len() {
        0 | 1 => [0.0, 1.0],
        len => [0.0, to_f64_usize(len - 1)],
    }
}

fn graph_y_bounds(history: &[u64]) -> [f64; 2] {
    let max_value = history.iter().copied().max().unwrap_or(0);
    [0.0, to_f64_u64(max_value.max(1))]
}

fn point_color(index: usize, value: u64, max_value: u64) -> Color {
    if index == 0 || value == 0 || max_value == 0 {
        return Color::LightGreen;
    }

    if value.saturating_mul(4) >= max_value.saturating_mul(3) {
        Color::Rgb(255, 165, 0)
    } else {
        Color::LightGreen
    }
}

fn should_highlight_pair(start: f64, end: f64, max_value: u64) -> bool {
    let max_value = to_f64_u64(max_value);
    if max_value == 0.0 {
        return false;
    }

    let start_high = start >= max_value * 0.75;
    let end_high = end >= max_value * 0.75;
    start_high || end_high
}

fn to_f64_usize(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn to_f64_u64(value: u64) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests {
    use super::{graph_x_bounds, graph_y_bounds, point_color, should_highlight_pair};
    use ratatui::style::Color;

    fn assert_bounds(actual: [f64; 2], expected: [f64; 2]) {
        assert!((actual[0] - expected[0]).abs() < f64::EPSILON);
        assert!((actual[1] - expected[1]).abs() < f64::EPSILON);
    }

    #[test]
    fn graph_x_bounds_use_single_unit_for_short_history() {
        assert_bounds(graph_x_bounds(&[]), [0.0, 1.0]);
        assert_bounds(graph_x_bounds(&[1]), [0.0, 1.0]);
    }

    #[test]
    fn graph_y_bounds_keep_zero_baseline_visible() {
        assert_bounds(graph_y_bounds(&[]), [0.0, 1.0]);
        assert_bounds(graph_y_bounds(&[0, 0]), [0.0, 1.0]);
        assert_bounds(graph_y_bounds(&[0, 12]), [0.0, 12.0]);
    }

    #[test]
    fn point_color_returns_green_for_low_values() {
        assert_eq!(point_color(1, 2, 10), Color::LightGreen);
    }

    #[test]
    fn point_color_returns_orange_for_high_values() {
        assert_eq!(point_color(1, 8, 10), Color::Rgb(255, 165, 0));
    }

    #[test]
    fn point_color_keeps_segment_start_green() {
        assert_eq!(point_color(0, 8, 10), Color::LightGreen);
    }

    #[test]
    fn should_highlight_pair_marks_segments_touching_high_values() {
        assert!(should_highlight_pair(2.0, 8.0, 10));
        assert!(!should_highlight_pair(2.0, 4.0, 10));
    }
}
