use gtk::{gio, prelude::*};
use crate::cache;
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;
use log::{info, warn};
use std::collections::HashSet;

pub struct Applications;

impl Applications {
    fn find_desktop_files() -> Vec<PathBuf> {
        let mut desktop_files = HashSet::new();
        let mut data_dirs = Vec::new();

        if let Ok(xdg_data_dirs) = env::var("XDG_DATA_DIRS") {
            info!("Using XDG_DATA_DIRS: {}", xdg_data_dirs);
            data_dirs.extend(xdg_data_dirs.split(':').map(String::from));
        } else {
            warn!("XDG_DATA_DIRS not set. Falling back to default paths.");
            if let Some(home_dir) = dirs::home_dir() {
                if let Some(local_share) = home_dir.join(".local/share").to_str() {
                    data_dirs.push(local_share.to_string());
                }
            }
            data_dirs.push("/usr/share".to_string());
            data_dirs.push("/usr/local/share".to_string());
            data_dirs.push("/run/current-system/sw/share/applications".to_string());
        }

        for data_dir in data_dirs {
            let path = PathBuf::from(data_dir);
            let app_dir = if path.file_name().and_then(|s| s.to_str()) == Some("applications") {
                path
            } else {
                path.join("applications")
            };

            if app_dir.is_dir() {
                info!("Scanning for applications in: {:?}", app_dir);
                for entry in WalkDir::new(&app_dir)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("desktop"))
                {
                    desktop_files.insert(entry.path().to_path_buf());
                }
            }
        }
        
        let mut sorted_files: Vec<_> = desktop_files.into_iter().collect();
        sorted_files.sort();
        sorted_files
    }

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

    pub fn scan_for_applications() -> Vec<String> {
        info!("Scanning system for applications.");
        let mut app_ids = HashSet::new();

        // Method 1: Standard GIO scan
        info!("Scanning with gio::AppInfo::all()");
        let all_apps = gio::AppInfo::all();
        for app_info in all_apps {
            if app_info.should_show() {
                if let Some(id) = app_info.id() {
                    app_ids.insert(id.to_string());
                }
            }
        }

        // Method 2: Manual scan of XDG_DATA_DIRS
        info!("Performing manual scan of XDG_DATA_DIRS.");
        let desktop_files = Self::find_desktop_files();
        for file_path in desktop_files {
            if let Some(app_info) = gio::DesktopAppInfo::from_filename(&file_path) {
                let app = app_info.upcast::<gio::AppInfo>();
                if app.should_show() {
                    if let Some(id) = app.id() {
                        app_ids.insert(id.to_string());
                    }
                }
            } else {
                warn!("Could not create AppInfo from file: {:?}", file_path);
            }
        }
        
        let mut sorted_app_ids: Vec<_> = app_ids.into_iter().collect();
        sorted_app_ids.sort();
        sorted_app_ids
    }
}