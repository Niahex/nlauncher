use gtk::{prelude::*, Application, ApplicationWindow, Entry};
use crate::state::LauncherState;
use crate::ui::{LauncherUi, build_ui};
use crate::events::connect_events;

pub struct Launcher {
    pub window: ApplicationWindow,
    ui: LauncherUi,
    state: LauncherState,
}

impl Launcher {
    pub fn new(app: &Application) -> Self {
        let search_entry_for_state = Entry::new();
        let search_entry_clone = search_entry_for_state.clone();

        let mut state = LauncherState::new(move || {
            search_entry_clone.text().to_string()
        });

        let mut ui = build_ui(app, &state);

        let real_search_entry = ui.search_entry.clone();
        state = LauncherState::new(move || {
            real_search_entry.text().to_string()
        });
        
        ui.list_view.set_model(Some(&state.selection_model));

        connect_events(&ui, &state);

        Self {
            window: ui.window.clone(),
            ui,
            state,
        }
    }

    pub fn init(&self) {
        // UI is already initialized in build_ui
    }

    pub fn show(&self) {
        self.window.present();
        self.ui.search_entry.grab_focus();

        if self.state.selection_model.n_items() > 0 {
            self.state.selection_model.set_selected(0);
        }
    }

    pub fn hide(&self) {
        self.ui.search_entry.set_text("");
        self.window.set_visible(false);
        self.state.app_filter.changed(gtk::FilterChange::Different);
    }
}
