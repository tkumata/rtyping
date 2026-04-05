use super::{App, MenuItem};

impl App {
    pub fn move_menu_up(&mut self) {
        self.toggle_menu_selection();
    }

    pub fn move_menu_down(&mut self) {
        self.toggle_menu_selection();
    }

    pub fn select_start_game(&mut self) {
        self.menu_selected = MenuItem::StartGame;
    }

    fn toggle_menu_selection(&mut self) {
        self.menu_selected = match self.menu_selected {
            MenuItem::StartGame => MenuItem::Config,
            MenuItem::Config => MenuItem::StartGame,
        };
    }
}
