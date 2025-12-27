use gtk::{gio, prelude::*};
use crate::cache;
use log::info;
use std::collections::HashSet;

pub struct Applications;

impl Applications {
    pub fn get_cached_applications() -> gio::ListStore {
        let app_list_store = gio::ListStore::new::<gio::AppInfo>();
        if let Some(app_ids) = cache::load_from_cache::<Vec<String>>() {
            info!("Loading applications from cache.");
            for id in app_ids {
                if let Some(desktop_app_info) = gio::DesktopAppInfo::new(&id) {
                    let app_info = desktop_app_info.upcast::<gio::AppInfo>();
                    app_list_store.append(&app_info);
                }
            }
        }
        app_list_store
    }

    pub fn scan_for_applications() -> HashSet<String> {
        info!("Scanning system for applications.");
        let mut app_ids = HashSet::new();

        // Standard GIO scan is sufficient as it covers all XDG_DATA_DIRS
        // and respects system configurations.
        let all_apps = gio::AppInfo::all();
        for app_info in all_apps {
            if app_info.should_show() {
                if let Some(id) = app_info.id() {
                    app_ids.insert(id.to_string());
                }
            }
        }

        app_ids
    }
}
