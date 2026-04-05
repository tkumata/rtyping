use super::{App, MenuItem};

impl App {
    const MENU_ITEMS: [MenuItem; 4] = [
        MenuItem::StartGame,
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
