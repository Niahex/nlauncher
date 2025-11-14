use crate::state::ApplicationInfo;
use freedesktop_desktop_entry::{DesktopEntry, Iter};
use freedesktop_icons::lookup;
use std::fs;
use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;

const CACHE_FILE: &str = "/tmp/nlauncher_cache.json";

pub fn load_applications() -> Vec<ApplicationInfo> {
    // Try to load from cache first
    if let Ok(cached) = load_from_cache() {
        return cached;
    }
    
    // Cache miss or invalid, scan applications
    let applications = scan_applications();
    
    // Save to cache
    let _ = save_to_cache(&applications);
    
    applications
}

fn load_from_cache() -> Result<Vec<ApplicationInfo>, Box<dyn std::error::Error>> {
    let cache_path = Path::new(CACHE_FILE);
    
    // Check if cache exists and is recent (less than 1 hour old)
    let metadata = fs::metadata(cache_path)?;
    let cache_age = SystemTime::now().duration_since(metadata.modified()?)?;
    if cache_age.as_secs() > 3600 {
        return Err("Cache too old".into());
    }
    
    let content = fs::read_to_string(cache_path)?;
    let applications: Vec<ApplicationInfo> = serde_json::from_str(&content)?;
    
    Ok(applications)
}

fn save_to_cache(applications: &[ApplicationInfo]) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string(applications)?;
    fs::write(CACHE_FILE, content)?;
    Ok(())
}

fn scan_applications() -> Vec<ApplicationInfo> {
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
