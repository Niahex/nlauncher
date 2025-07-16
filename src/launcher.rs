use gtk::{prelude::*, Application, ApplicationWindow};
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
        let ui = build_ui(app);
        let state = {
            let search_entry = ui.search_entry.clone();
            LauncherState::new(move || search_entry.text().to_string())
        };

        ui.list_view.set_model(Some(&state.selection_model));
        connect_events(&ui, &state, app);

        Self {
            window: ui.window.clone(),
            ui,
            state,
        }
    }

    pub fn init(&self) {}

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
