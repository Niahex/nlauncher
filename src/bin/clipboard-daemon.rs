use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

const MAX_HISTORY: usize = 100;

fn get_history_path() -> PathBuf {
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .unwrap_or_else(|_| format!("{}/.cache", std::env::var("HOME").unwrap()));
    PathBuf::from(cache_dir)
        .join("nlauncher")
        .join("clipboard_history.txt")
}

fn get_current_clipboard() -> Option<String> {
    Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .filter(|s| !s.is_empty())
}

fn load_history() -> Vec<String> {
    let path = get_history_path();
    if let Ok(content) = fs::read_to_string(&path) {
        content.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

fn save_history(history: &[String]) {
    let path = get_history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
    {
        for entry in history {
            let _ = writeln!(file, "{entry}");
        }
    }
}

fn main() {
    eprintln!("[clipboard-daemon] Starting clipboard monitor...");

    let mut last_content = String::new();
    let mut history = load_history();

    loop {
        if let Some(content) = get_current_clipboard() {
            if content != last_content && !content.is_empty() {
                eprintln!("[clipboard-daemon] New clipboard content detected");

                // Remove if already exists
                history.retain(|s| s != &content);

                // Add to front
                history.insert(0, content.clone());

                // Keep only MAX_HISTORY items
                if history.len() > MAX_HISTORY {
                    history.truncate(MAX_HISTORY);
                }

                save_history(&history);
                last_content = content;
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}
