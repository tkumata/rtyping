mod common;
mod config_screen;
mod loading;
mod menu;
mod result;
mod typing;
mod wpm_graph;

use ratatui::Frame;

use super::app::{App, AppState};

pub fn help_line_count() -> u16 {
    common::help_line_count()
}

pub fn render(frame: &mut Frame, app: &App) {
    match app.state() {
        AppState::Menu => menu::render_menu(frame, app),
        AppState::Config => config_screen::render_config(frame, app),
        AppState::Loading => loading::render_loading(frame, app),
        AppState::Typing => typing::render_typing(frame, app),
        AppState::Result => result::render_result(frame, app),
    }
}
