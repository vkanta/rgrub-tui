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

    pub fn select_menu_item(&mut self) {
        if let Some(item) = self.menu_items.get(self.menu_selection) {
            match item {
                crate::model::entry::GrubMenu::Entry(_) => {
                    self.current_view = View::EntryDetailView;
                }
                crate::model::entry::GrubMenu::Submenu { .. } => {}
            }
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
        keys.get(self.settings_selection).cloned().map(|s| s.to_string())
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
            _ => self.current_view = View::MenuView,
        }
    }
}
