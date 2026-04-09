use super::{App, MenuItem};

impl App {
    const MENU_ITEMS: [MenuItem; 5] = [
        MenuItem::StartGame,
        MenuItem::PracticeMode,
        MenuItem::StartGameGoogle,
        MenuItem::StartGameGroq,
        MenuItem::Config,
    ];

    pub fn move_menu_up(&mut self) {
        self.menu_selected = self.cycle_menu_selection(-1);
    }

    pub fn move_menu_down(&mut self) {
        self.menu_selected = self.cycle_menu_selection(1);
    }

    pub fn select_start_game(&mut self) {
        self.menu_selected = MenuItem::StartGame;
    }

    fn cycle_menu_selection(&self, delta: isize) -> MenuItem {
        let current_index = Self::MENU_ITEMS
            .iter()
            .position(|item| *item == self.menu_selected)
            .unwrap_or(0) as isize;
        let len = Self::MENU_ITEMS.len() as isize;
        let next_index = (current_index + delta).rem_euclid(len) as usize;
        Self::MENU_ITEMS[next_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::usecase::generate_sentence::GenerationSource;

    fn new_app() -> App {
        App::new(
            60,
            30,
            80.0,
            false,
            GenerationSource::Local,
            AppConfig::default(),
        )
    }

    #[test]
    fn return_to_menu_with_start_selected_clears_practice_mode() {
        let mut app = new_app();

        app.set_practice_mode(true);
        app.return_to_menu_with_start_selected();

        assert_eq!(app.menu_selected(), MenuItem::StartGame);
        assert!(!app.is_practice_mode());
    }
}
