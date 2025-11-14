use crate::state::ApplicationInfo;
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use freedesktop_icons::lookup;
use std::fs;
use std::collections::HashSet;

pub fn load_applications() -> Vec<ApplicationInfo> {
    let mut applications = Vec::new();
    let mut seen_names = HashSet::new();
    
    for path in Iter::new(freedesktop_desktop_entry::default_paths()) {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(desktop_entry) = DesktopEntry::decode(&path, &content) {
                if let Some(name) = desktop_entry.name(None) {
                    if let Some(exec) = desktop_entry.exec() {
                        // Skip if we've already seen this name
                        if seen_names.contains(&name.to_string()) {
                            continue;
                        }
                        seen_names.insert(name.to_string());
                        
                        let icon_path = desktop_entry.icon()
                            .and_then(|icon_name| lookup(icon_name).with_size(24).find())
                            .map(|p| p.to_string_lossy().to_string());
                        
                        applications.push(ApplicationInfo {
                            name: name.to_string(),
                            exec: exec.to_string(),
                            icon: desktop_entry.icon().map(|s| s.to_string()),
                            icon_path,
                        });
                    }
                }
            }
        }
    }
    
    applications.sort_by(|a, b| a.name.cmp(&b.name));
    applications
}
