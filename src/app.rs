#[derive(PartialEq, Clone, Copy)]
pub enum View {
    MenuView,
    SettingsView,
    EntryDetailView,
    EditingSettings,
    ConfirmSave,
}

pub struct App {
    pub current_view: View,
    pub settings: crate::model::settings::GrubSettings,
    pub menu_items: Vec<crate::model::entry::GrubMenu>,
    pub menu_stack: Vec<Vec<crate::model::entry::GrubMenu>>, // Track navigation for submenus
    pub menu_selection: usize,
    pub settings_selection: usize,
    pub should_quit: bool,

    // Editing state
    pub editing_key: Option<String>,
    pub edit_buffer: String,
}

impl App {
    pub fn new(settings: crate::model::settings::GrubSettings, menu_items: Vec<crate::model::entry::GrubMenu>) -> Self {
        Self {
            current_view: View::MenuView,
            settings,
            menu_items,
            menu_stack: Vec::new(),
            menu_selection: 0,
            settings_selection: 0,
            should_quit: false,
            editing_key: None,
            edit_buffer: String::new(),
        }
    }

    pub fn next_menu_item(&mut self) {
        if self.menu_selection < self.menu_items.len().saturating_sub(1) {
            self.menu_selection += 1;
        }
    }

    pub fn prev_menu_item(&mut self) {
        if self.menu_selection > 0 {
            self.menu_selection -= 1;
        }
    }

    pub fn next_settings_item(&mut self) {
        let len = self.settings.entries.len();
        if len > 0 && self.settings_selection < len - 1 {
            self.settings_selection += 1;
        }
    }

    pub fn prev_settings_item(&mut self) {
        if self.settings_selection > 0 {
            self.settings_selection -= 1;
        }
    }

    // Move entry UP
    pub fn move_menu_item_up(&mut self) {
        if self.menu_selection > 0 {
            self.menu_items.swap(self.menu_selection, self.menu_selection - 1);
            self.menu_selection -= 1;
        }
    }

    // Move entry DOWN
    pub fn move_menu_item_down(&mut self) {
        if self.menu_selection < self.menu_items.len().saturating_sub(1) {
            self.menu_items.swap(self.menu_selection, self.menu_selection + 1);
            self.menu_selection += 1;
        }
    }

    // Set default entry
    pub fn set_default_entry(&mut self) {
        if let Some(item) = self.menu_items.get(self.menu_selection) {
             match item {
                 crate::model::entry::GrubMenu::Entry(e) => {
                     self.settings.set("GRUB_DEFAULT".to_string(), e.name.clone());
                 }
                 _ => {}
             }
        }
    }

    pub fn select_menu_item(&mut self) {
        // To avoid the borrow checker error, we first check if it's a submenu
        // without holding an immutable reference to 'self.menu_items'
        let is_submenu = if let Some(crate::model::entry::GrubMenu::Submenu { .. }) = self.menu_items.get(self.menu_selection) {
            true
        } else {
            false
        };

        if is_submenu {
            // Now it's safe to mutably borrow because we are no longer holding 'item'
            if let Some(crate::model::entry::GrubMenu::Submenu { entries, .. }) = self.menu_items.get(self.menu_selection).cloned() {
                let old_items = std::mem::replace(&mut self.menu_items, entries);
                self.menu_stack.push(old_items);
                self.menu_selection = 0;
            }
        } else if let Some(crate::model::entry::GrubMenu::Entry(_)) = self.menu_items.get(self.menu_selection) {
            self.current_view = View::EntryDetailView;
        }
    }

    pub fn toggle_view(&mut self) {
        self.current_view = match self.current_view {
            View::MenuView => View::SettingsView,
            View::SettingsView => View::MenuView,
            View::EntryDetailView => View::MenuView,
            _ => View::MenuView,
        };
        self.menu_selection = 0;
        self.settings_selection = 0;
    }

    pub fn enter_edit_mode(&mut self) {
        if let Some(key) = self.get_selected_setting_key() {
            let val = self.settings.get(&key).cloned().unwrap_or_default();
            self.editing_key = Some(key);
            self.edit_buffer = val;
            self.current_view = View::EditingSettings;
        }
    }

    pub fn get_selected_setting_key(&self) -> Option<String> {
        let keys: Vec<_> = self.settings.entries.keys().collect();
        keys.get(self.settings_selection).map(|&k| k.to_string())
    }

    pub fn apply_edit(&mut self) {
        if let Some(key) = self.editing_key.take() {
            self.settings.set(key, self.edit_buffer.clone());
            self.current_view = View::SettingsView;
        }
    }

    pub fn cancel_edit(&mut self) {
        self.editing_key = None;
        self.edit_buffer.clear();
        self.current_view = View::SettingsView;
    }

    pub fn request_save(&mut self) {
        self.current_view = View::ConfirmSave;
    }

    pub fn go_back(&mut self) {
        match self.current_view {
            View::EntryDetailView | View::EditingSettings | View::ConfirmSave => {
                self.current_view = View::SettingsView; 
            },
            View::MenuView => {
                if let Some(prev_menu) = self.menu_stack.pop() {
                    self.menu_items = prev_menu;
                    self.menu_selection = 0;
                }
            },
            _ => self.current_view = View::MenuView,
        }
    }
}
