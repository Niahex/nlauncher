use gtk::{gio, prelude::*};
use crate::cache;

pub struct Applications;

impl Applications {
    pub fn get_all_applications() -> gio::ListStore {
        let app_list_store = gio::ListStore::new::<gio::AppInfo>();

        // Essayer de charger depuis le cache
        if let Some(app_ids) = cache::load_from_cache::<Vec<String>>() {
            for id in app_ids {
                if let Some(desktop_app_info) = gio::DesktopAppInfo::new(&id) {
                    // Convertir DesktopAppInfo en AppInfo
                    let app_info = desktop_app_info.upcast::<gio::AppInfo>();
                    app_list_store.append(&app_info);
                }
            }
        } else {
            // Si le cache est vide ou invalide, scanner le système
            let all_apps = gio::AppInfo::all();
            let mut app_ids = Vec::new();
            for app_info in all_apps {
                if let Some(id) = app_info.id() {
                    app_ids.push(id.to_string());
                    app_list_store.append(&app_info);
                }
            }
            // Sauvegarder la nouvelle liste dans le cache pour la prochaine fois
            if let Err(e) = cache::save_to_cache(&app_ids) {
                eprintln!("Failed to save app cache: {}", e);
            }
        }

        app_list_store
    }
}
