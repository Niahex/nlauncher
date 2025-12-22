use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct ClipboardEntry {
    pub content: String,
}

fn get_history_path() -> PathBuf {
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .unwrap_or_else(|_| format!("{}/.cache", std::env::var("HOME").unwrap()));
    PathBuf::from(cache_dir)
        .join("nlauncher")
        .join("clipboard_history.txt")
}

pub fn get_clipboard_history() -> Vec<ClipboardEntry> {
    let path = get_history_path();
    if let Ok(content) = fs::read_to_string(&path) {
        content
            .lines()
            .map(|line| ClipboardEntry {
                content: line.to_string(),
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn set_clipboard(content: &str) -> Result<(), std::io::Error> {
    Command::new("wl-copy").arg(content).spawn()?;
    Ok(())
}

pub fn is_clipboard_query(query: &str) -> bool {
    query.starts_with("clip")
}
