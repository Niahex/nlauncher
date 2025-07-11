use gtk::gio;

pub struct Applications;

impl Applications {
    pub fn get_all_applications() -> gio::ListStore {
        let app_list_store = gio::ListStore::new::<gio::AppInfo>();
        gio::AppInfo::all().iter().for_each(|app_info| {
            app_list_store.append(&app_info.clone());
        });
        app_list_store
    }
}
