use super::{App, MenuItem};

impl App {
    pub fn visible_menu_items(&self) -> Vec<MenuItem> {
        let mut items = vec![MenuItem::StartGame, MenuItem::PracticeMode];
        if self.config.google.is_ready() {
            items.push(MenuItem::StartGameGoogle);
        }
        if self.config.groq.is_ready() {
            items.push(MenuItem::StartGameGroq);
        }
        items.extend([MenuItem::Stats, MenuItem::Config]);
        items
    }

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
        let menu_items = self.visible_menu_items();
        #[expect(clippy::cast_possible_wrap)]
        let current_index = menu_items
            .iter()
            .position(|item| *item == self.menu_selected)
            .unwrap_or(0) as isize;
        let len = menu_items.len().cast_signed();
        let next_index = (current_index + delta).rem_euclid(len) as usize;
        menu_items
            .get(next_index)
            .copied()
            .unwrap_or(MenuItem::StartGame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;

    fn new_app() -> App {
        App::new(AppConfig::default())
    }

    #[test]
    fn return_to_menu_with_start_selected_clears_practice_mode() {
        let mut app = new_app();

        app.set_practice_mode(true);
        app.return_to_menu_with_start_selected();

        assert_eq!(app.menu_selected(), MenuItem::StartGame);
        assert!(!app.is_practice_mode());
    }

    #[test]
    fn visible_menu_items_hide_incomplete_provider_entries() {
        let app = new_app();

        assert_eq!(
            app.visible_menu_items(),
            vec![
                MenuItem::StartGame,
                MenuItem::PracticeMode,
                MenuItem::Stats,
                MenuItem::Config,
            ]
        );
    }

    #[test]
    fn visible_menu_items_include_ready_provider_entries() {
        let mut app = new_app();
        app.config.google.api_url = "https://google.example".to_string();
        app.config.google.api_key = "google-key".to_string();
        app.config.google.model = "google-model".to_string();
        app.config.groq.api_url = "https://groq.example".to_string();
        app.config.groq.api_key = "groq-key".to_string();
        app.config.groq.model = "groq-model".to_string();

        assert_eq!(
            app.visible_menu_items(),
            vec![
                MenuItem::StartGame,
                MenuItem::PracticeMode,
                MenuItem::StartGameGoogle,
                MenuItem::StartGameGroq,
                MenuItem::Stats,
                MenuItem::Config,
            ]
        );
    }

    #[test]
    fn menu_navigation_skips_incomplete_provider_entries() {
        let mut app = new_app();

        app.move_menu_down();
        assert_eq!(app.menu_selected(), MenuItem::PracticeMode);

        app.move_menu_down();
        assert_eq!(app.menu_selected(), MenuItem::Stats);
    }
}
