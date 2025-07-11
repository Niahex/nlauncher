use gtk::{prelude::*, glib, gio, Application, ApplicationWindow, SingleSelection, EventControllerKey, gdk, ListView};
use crate::ui::LauncherUi;
use crate::state::LauncherState;

pub fn connect_events(ui: &LauncherUi, state: &LauncherState, app: &Application) {
    let app_filter = state.app_filter.clone();
    let selection_model = state.selection_model.clone();
    ui.search_entry.connect_changed(move |_| {
        app_filter.changed(gtk::FilterChange::Different);
        let selection_model = selection_model.clone();
        glib::idle_add_local_once(move || {
            if selection_model.n_items() > 0 {
                selection_model.set_selected(0);
            }
        });
    });

    let window = ui.window.clone();
    let selection_model = state.selection_model.clone();
    ui.search_entry.connect_activate(move |_| {
        launch_selected_app(&window, &selection_model);
    });

    let key_controller = EventControllerKey::new();
    let app_clone = app.clone();
    let list_view_clone = ui.list_view.clone();
    let selection_model_clone = state.selection_model.clone();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
            match keyval {
                gdk::Key::Escape => {
                    app_clone.quit();
                    glib::Propagation::Stop
                },
                gdk::Key::Down => {
                    navigate_list(&selection_model_clone, &list_view_clone, 1);
                    glib::Propagation::Stop
                },
                gdk::Key::Up => {
                    navigate_list(&selection_model_clone, &list_view_clone, -1);
                    glib::Propagation::Stop
                },
                _ => glib::Propagation::Proceed,
            }
        }
    );
    ui.window.add_controller(key_controller);

    let search_entry_clone = ui.search_entry.clone();
    ui.window.connect_unrealize(move |_| {
        search_entry_clone.set_text("");
    });
}

fn navigate_list(selection_model: &SingleSelection, list_view: &ListView, direction: i32) {
    let current_pos = selection_model.selected();
    let n_items = selection_model.n_items();
    if n_items == 0 {
        return;
    }

    let next_pos = if current_pos == gtk::INVALID_LIST_POSITION {
        if direction > 0 { 0 } else { n_items - 1 }
    } else {
        (current_pos as i32 + direction + n_items as i32) as u32 % n_items
    };

    selection_model.set_selected(next_pos);
    list_view.scroll_to(next_pos, gtk::ListScrollFlags::SELECT, None);
}

pub fn launch_selected_app(window: &ApplicationWindow, selection_model: &SingleSelection) -> bool {
    if let Some(selected_item) = selection_model.selected_item() {
        if let Some(app_info) = selected_item.downcast_ref::<gio::AppInfo>() {
            match app_info.launch(&[], gio::AppLaunchContext::NONE) {
                Ok(_) => {
                    window.set_visible(false);
                    return true;
                },
                Err(e) => {
                    eprintln!("Error launching application: {}", e);
                    return false;
                }
            }
        }
    }
    false
}

