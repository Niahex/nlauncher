use gpui::{
    actions, background_executor, div, img, layer_shell::*, point, prelude::*, px, rgb, rgba, App,
    Application, Bounds, Context, FocusHandle, KeyBinding, KeyDownEvent, Render, SharedString,
    Size, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod applications;
mod calculator;
mod fuzzy;
mod process;
mod state;
mod vault;
use applications::{load_from_cache, save_to_cache, scan_applications};
use calculator::{is_calculator_query, Calculator};
use fuzzy::FuzzyMatcher;
use process::{get_running_processes, is_process_query, kill_process, ProcessInfo};
use state::ApplicationInfo;
use vault::{VaultEntry, VaultManager};

actions!(launcher, [Quit, Backspace, Up, Down, Launch, CopyPassword, CopyUsername, CopyTotp, OpenSettings]);

enum SearchResult {
    Application(usize),
    Calculation(String),
    Process(ProcessInfo),
    VaultEntry(VaultEntry),
}

struct Launcher {
    focus_handle: FocusHandle,
    applications: Vec<ApplicationInfo>,
    query: SharedString,
    selected_index: usize,
    fuzzy_matcher: FuzzyMatcher,
    calculator: Option<Calculator>,
    search_results: Vec<SearchResult>,
    scroll_offset: usize,
    vault_manager: VaultManager,
    vault_unlocking: bool,
    vault_entries: Vec<VaultEntry>,
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {
        let vault_manager = VaultManager::new();
        
        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications: Vec::new(),
            query: "".into(),
            selected_index: 0,
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: None,
            search_results: Vec::new(),
            scroll_offset: 0,
            vault_manager: vault_manager.clone(),
            vault_unlocking: false,
            vault_entries: Vec::new(),
        };

        // Try to load vault from existing session in background
        if vault_manager.is_unlocked() {
            cx.spawn(async move |this, cx| {
                let result = background_executor().spawn(async move {
                    vault_manager.load_from_session()
                }).await;
                
                if let Ok(entries) = result {
                    this.update(cx, |this, _cx| {
                        this.vault_entries = entries;
                    })?;
                }
                anyhow::Ok(())
            }).detach();
        }

        if let Some(apps) = load_from_cache() {
            launcher.applications = apps;
            launcher.update_search_results();
        }

        cx.spawn(async move |this, cx| {
            let apps = background_executor()
                .spawn(async move { scan_applications() })
                .await;

            this.update(cx, |this, cx| {
                this.applications = apps.clone();
                this.update_search_results();
                cx.notify();
            })?;

            background_executor()
                .spawn(async move {
                    let _ = save_to_cache(&apps);
                })
                .detach();

            anyhow::Ok(())
        })
        .detach();

        cx.spawn(async move |this, cx| {
            let calculator = background_executor()
                .spawn(async move { Calculator::new() })
                .await;

            this.update(cx, |this, cx| {
                this.calculator = Some(calculator);
                if is_calculator_query(&this.query) {
                    this.update_search_results();
                    cx.notify();
                }
            })?;

            anyhow::Ok(())
        })
        .detach();

        launcher
    }

    fn update_search_results(&mut self) {
        let query_str = self.query.to_string();
        self.search_results.clear();

        if query_str.starts_with("pass") && query_str.len() > 4 {
            let rest = query_str.strip_prefix("pass").unwrap_or("");
            
            // If vault is empty but session exists, show loading message
            if self.vault_entries.is_empty() && self.vault_manager.is_unlocked() {
                self.search_results.push(SearchResult::Calculation("Loading vault...".to_string()));
            } else if !self.vault_entries.is_empty() {
                // Filter cached entries
                let search_lower = rest.to_lowercase();
                for entry in &self.vault_entries {
                    if search_lower.is_empty() 
                        || entry.title.to_lowercase().contains(&search_lower)
                        || entry.username.to_lowercase().contains(&search_lower) {
                        self.search_results.push(SearchResult::VaultEntry(entry.clone()));
                    }
                }
            }
        } else {
            // Clear vault entries when leaving vault mode
            self.vault_entries.clear();
            
            if is_process_query(&query_str) {
                let processes = get_running_processes();
                if query_str == "ps" {
                    for process in processes {
                        self.search_results.push(SearchResult::Process(process));
                    }
                } else if query_str.starts_with("ps") && query_str.len() > 2 {
                    let search_term = query_str.strip_prefix("ps").unwrap_or("").to_lowercase();
                    for process in processes {
                        if process.name.to_lowercase().contains(&search_term) {
                            self.search_results.push(SearchResult::Process(process));
                        }
                    }
                }
            } else if is_calculator_query(&query_str) {
                if let Some(calculator) = &mut self.calculator {
                    if let Some(result) = calculator.evaluate(&query_str) {
                        self.search_results.push(SearchResult::Calculation(result));
                    }
                } else {
                    self.search_results.push(SearchResult::Calculation(
                        "Initializing calculator...".to_string(),
                    ));
                }
            } else {
                let app_indices = self.fuzzy_matcher.search(&query_str, &self.applications);
                for index in app_indices.into_iter().take(50) {
                    self.search_results.push(SearchResult::Application(index));
                }
            }
        }

        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let mut query = self.query.to_string();
        if !query.is_empty() {
            query.pop();
            self.query = query.into();
            self.update_search_results();
            cx.notify();
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        if !self.search_results.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
            cx.notify();
        }
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if !self.search_results.is_empty() && self.selected_index + 1 < self.search_results.len() {
            self.selected_index += 1;
            let visible_items = 10;
            if self.selected_index >= self.scroll_offset + visible_items {
                self.scroll_offset = self.selected_index - visible_items + 1;
            }
            cx.notify();
        }
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        let query_str = self.query.to_string();
        
        // Handle "pass{password}" - unlock vault on Enter
        if query_str.starts_with("pass") && query_str.len() > 4 {
            let rest = query_str.strip_prefix("pass").unwrap_or("");
            if !rest.is_empty() && !self.vault_manager.is_unlocked() {
                if self.vault_unlocking {
                    return; // Already unlocking
                }
                
                self.vault_unlocking = true;
                self.search_results.clear();
                self.search_results.push(SearchResult::Calculation("Unlocking vault...".to_string()));
                cx.notify();
                
                let password = rest.to_string();
                let vault_manager = self.vault_manager.clone();
                
                cx.spawn(async move |this, cx| {
                    let result = background_executor().spawn(async move {
                        vault_manager.unlock(&password)
                    }).await;
                    
                    this.update(cx, |this, cx| {
                        this.vault_unlocking = false;
                        match result {
                            Ok(entries) => {
                                this.vault_entries = entries;
                                this.query = "pass".into();
                                this.update_search_results();
                                this.selected_index = 0;
                            }
                            Err(e) => {
                                this.search_results.clear();
                                this.search_results.push(SearchResult::Calculation(
                                    format!("❌ Wrong password or vault error: {}", e)
                                ));
                            }
                        }
                        cx.notify();
                    })
                }).detach();
                return;
            }
        }
        
        if let Some(result) = self.search_results.get(self.selected_index) {
            match result {
                SearchResult::Application(app_index) => {
                    if let Some(app) = self.applications.get(*app_index) {
                        let exec = app.exec.clone();
                        let name = app.name.clone();

                        std::thread::spawn(move || {
                            let mut cmd = Command::new("sh");
                            cmd.arg("-c")
                                .arg(&exec)
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null());

                            match cmd.spawn() {
                                Ok(_) => eprintln!("[nlauncher] Launched: {name}"),
                                Err(err) => eprintln!("[nlauncher] Failed to launch {name} (exec: {exec}): {err}"),
                            }
                        });

                        cx.quit();
                    }
                }
                SearchResult::Calculation(result) => {
                    if result != "Initializing calculator..." {
                        match std::process::Command::new("wl-copy").arg(result).output() {
                            Ok(_) => eprintln!("[nlauncher] Copied to clipboard: {result}"),
                            Err(e) => eprintln!("[nlauncher] Failed to copy to clipboard: {e}"),
                        }
                        cx.quit();
                    }
                }
                SearchResult::Process(process) => {
                    let pid = process.pid;
                    let name = process.name.clone();
                    std::thread::spawn(move || {
                        match kill_process(pid) {
                            Ok(_) => eprintln!("[nlauncher] Killed process: {name} (pid: {pid})"),
                            Err(e) => eprintln!("[nlauncher] Failed to kill process {name} (pid: {pid}): {e}"),
                        }
                    });

                    self.update_search_results();
                    if self.selected_index >= self.search_results.len() && self.selected_index > 0 {
                        self.selected_index = self.search_results.len().saturating_sub(1);
                    }
                    cx.notify();
                }
                SearchResult::VaultEntry(_) => {
                    // Do nothing on Enter for vault entries
                }
            }
        }
    }

    fn copy_password(&mut self, _: &CopyPassword, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(SearchResult::VaultEntry(entry)) = self.search_results.get(self.selected_index) {
            let password = entry.password.clone();
            let title = entry.title.clone();
            
            eprintln!("[nlauncher] Copying password for: {}", title);
            
            let result = std::process::Command::new("wl-copy")
                .arg(&password)
                .spawn();
            
            match result {
                Ok(_) => eprintln!("[nlauncher] wl-copy spawned successfully"),
                Err(e) => eprintln!("[nlauncher] Failed to spawn wl-copy: {}", e),
            }
            
            // Wait a bit before quitting
            std::thread::sleep(std::time::Duration::from_millis(100));
            cx.quit();
        }
    }

    fn copy_username(&mut self, _: &CopyUsername, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(SearchResult::VaultEntry(entry)) = self.search_results.get(self.selected_index) {
            let username = entry.username.clone();
            let title = entry.title.clone();
            
            eprintln!("[nlauncher] Copying username for: {}", title);
            
            let _ = std::process::Command::new("wl-copy")
                .arg(&username)
                .spawn();
            
            std::thread::sleep(std::time::Duration::from_millis(100));
            cx.quit();
        }
    }

    fn copy_totp(&mut self, _: &CopyTotp, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(SearchResult::VaultEntry(entry)) = self.search_results.get(self.selected_index) {
            if let Some(totp) = &entry.totp {
                let totp = totp.clone();
                let title = entry.title.clone();
                
                eprintln!("[nlauncher] Copying TOTP for: {}", title);
                
                let _ = std::process::Command::new("wl-copy")
                    .arg(&totp)
                    .spawn();
                
                std::thread::sleep(std::time::Duration::from_millis(100));
                cx.quit();
            }
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_index = self.selected_index;
        let query_text = self.query.to_string();
        let focus_handle = self.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "space" {
                    let mut query = this.query.to_string();
                    query.push(' ');
                    
                    // Trigger vault reload if entering pass mode with active session
                    let should_reload = query == "pass" && this.vault_entries.is_empty() && this.vault_manager.is_unlocked();
                    
                    this.query = query.into();
                    this.update_search_results();
                    
                    if should_reload {
                        let vault_manager = this.vault_manager.clone();
                        cx.spawn(async move |this, cx| {
                            let result = background_executor().spawn(async move {
                                vault_manager.load_from_session()
                            }).await;
                            
                            if let Ok(entries) = result {
                                this.update(cx, |this, cx| {
                                    this.vault_entries = entries;
                                    this.update_search_results();
                                    cx.notify();
                                })?;
                            }
                            anyhow::Ok(())
                        }).detach();
                    }
                    
                    cx.notify();
                } else if let Some(key_char) = &event.keystroke.key_char {
                    let is_password_mode = this.query.starts_with("pass") && this.query.len() > 4 && !this.vault_manager.is_unlocked();
                    let allowed = if is_password_mode {
                        true
                    } else {
                        key_char.chars().all(|c| c.is_alphanumeric() || "+-*/()^.=".contains(c))
                    };
                    
                    if allowed {
                        let mut query = this.query.to_string();
                        query.push_str(key_char);
                        this.query = query.into();
                        this.update_search_results();
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::launch))
            .on_action(cx.listener(Self::copy_password))
            .on_action(cx.listener(Self::copy_username))
            .on_action(cx.listener(Self::copy_totp))
            .child(
                div()
                    .w(px(700.))
                    .max_h(px(500.))
                    .bg(rgba(0x2e3440dd))
                    .border_1()
                    .border_color(rgba(0x88c0d033))
                    .rounded_lg()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(
                        div()
                            .p_2()
                            .bg(rgb(0x3b4252))
                            .rounded_md()
                            .flex()
                            .gap_1()
                            .text_color(if query_text.is_empty() {
                                rgba(0xd8dee966)
                            } else {
                                rgb(0xeceff4)
                            })
                            .child(if query_text.is_empty() {
                                div().child("Search for apps and commands")
                            } else if query_text.starts_with("ps") {
                                let (cmd, rest) = if query_text.starts_with("ps") && query_text.len() > 2 {
                                    ("ps".to_string(), query_text.strip_prefix("ps").unwrap_or("").to_string())
                                } else if query_text == "ps" {
                                    ("ps".to_string(), String::new())
                                } else {
                                    (String::new(), query_text.clone())
                                };
                                
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(rgb(0xbf616a))
                                            .rounded_sm()
                                            .child(cmd)
                                    )
                                    .child(rest)
                            } else if query_text.starts_with("pass") {
                                let (cmd, rest) = if query_text.starts_with("pass") && query_text.len() > 4 {
                                    ("pass".to_string(), query_text.strip_prefix("pass").unwrap_or("").to_string())
                                } else if query_text == "pass" {
                                    ("pass".to_string(), String::new())
                                } else {
                                    (String::new(), query_text.clone())
                                };
                                
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(rgb(0xebcb8b))
                                            .rounded_sm()
                                            .child(cmd)
                                    )
                                    .child(rest)
                            } else if query_text.starts_with('=') {
                                let rest = query_text.strip_prefix('=').unwrap_or("").to_string();
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(rgb(0xa3be8c))
                                            .rounded_sm()
                                            .child("=")
                                    )
                                    .child(rest)
                            } else if query_text.starts_with("pass ") {
                                div().child(query_text.clone())
                            } else {
                                div().child(query_text.clone())
                            }),
                    )
                    .child(div().flex().flex_col().mt_2().children({
                        let visible_items = 10;
                        self.search_results
                            .iter()
                            .enumerate()
                            .skip(self.scroll_offset)
                            .take(visible_items)
                            .map(|(original_index, result)| {
                                let mut item = div()
                                    .flex()
                                    .items_center()
                                    .p_2()
                                    .text_color(rgb(0xd8dee9)) // snow1
                                    .rounded_md()
                                    .hover(|style| style.bg(rgb(0x434c5e)));

                                if original_index == selected_index {
                                    item = item.bg(rgba(0x88c0d033)).text_color(rgb(0x88c0d0));
                                }

                                match result {
                                    SearchResult::Application(app_index) => {
                                        if let Some(app) = self.applications.get(*app_index) {
                                            item.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(
                                                        if let Some(icon_path) = &app.icon_path {
                                                            div().size_6().child(
                                                                img(std::path::PathBuf::from(
                                                                    icon_path,
                                                                ))
                                                                .size_6(),
                                                            )
                                                        } else {
                                                            div()
                                                                .size_6()
                                                                .bg(rgb(0x5e81ac))
                                                                .rounded_sm()
                                                        },
                                                    )
                                                    .child(app.name.clone()),
                                            )
                                        } else {
                                            item.child("Invalid app")
                                        }
                                    }
                                    SearchResult::Calculation(calc_result) => item.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .size_6()
                                                    .bg(rgb(0xa3be8c))
                                                    .rounded_sm()
                                                    .child("="),
                                            )
                                            .child(format!("= {calc_result}")),
                                    ),
                                    SearchResult::Process(process) => item.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .size_6()
                                                    .bg(rgb(0xbf616a))
                                                    .rounded_sm()
                                                    .child("⚡"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(format!(
                                                        "{} ({})",
                                                        process.name, process.pid
                                                    ))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0x88c0d0))
                                                            .child(format!(
                                                                "CPU: {:.1}% | RAM: {:.1}MB",
                                                                process.cpu_usage,
                                                                process.memory_mb
                                                            )),
                                                    ),
                                            ),
                                    ),
                                    SearchResult::VaultEntry(entry) => item.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .size_6()
                                                    .bg(rgb(0xebcb8b))
                                                    .rounded_sm()
                                                    .child("🔑"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(format!("{} ({})", entry.title, entry.username))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0x88c0d0))
                                                            .child(if entry.totp.is_some() {
                                                                "Ctrl+C: password | Ctrl+B: username | Ctrl+T: TOTP"
                                                            } else {
                                                                "Ctrl+C: password | Ctrl+B: username"
                                                            }),
                                                    ),
                                            ),
                                    ),
                                }
                            })
                    })),
            )
    }
}

fn get_lock_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("nlauncher.lock")
}

fn is_running() -> bool {
    let lock_path = get_lock_path();
    if !lock_path.exists() {
        return false;
    }

    if let Ok(pid_str) = fs::read_to_string(&lock_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            return std::path::Path::new(&format!("/proc/{pid}")).exists();
        }
    }
    false
}

fn create_lock() {
    let lock_path = get_lock_path();
    let pid = std::process::id();
    let _ = fs::write(lock_path, pid.to_string());
}

fn remove_lock() {
    let lock_path = get_lock_path();
    let _ = fs::remove_file(lock_path);
}

fn main() {
    if is_running() {
        if let Ok(pid_str) = fs::read_to_string(get_lock_path()) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
            }
        }
        std::process::exit(0);
    }

    create_lock();

    let _ = std::panic::catch_unwind(|| {
        Application::new().run(|cx: &mut App| {
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.bind_keys([
                KeyBinding::new("backspace", Backspace, None),
                KeyBinding::new("up", Up, None),
                KeyBinding::new("down", Down, None),
                KeyBinding::new("enter", Launch, None),
                KeyBinding::new("escape", Quit, None),
                KeyBinding::new("ctrl-c", CopyPassword, None),
                KeyBinding::new("ctrl-b", CopyUsername, None),
                KeyBinding::new("ctrl-t", CopyTotp, None),
            ]);

            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: Size::new(px(800.), px(600.)),
                        })),
                        app_id: Some("nlauncher".to_string()),
                        window_background: WindowBackgroundAppearance::Transparent,
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "nlauncher".to_string(),
                            anchor: Anchor::empty(),
                            margin: Some((px(0.), px(0.), px(0.), px(0.))),
                            keyboard_interactivity: KeyboardInteractivity::Exclusive,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    |_, cx| cx.new(Launcher::new),
                )
                .unwrap();

            window
                .update(cx, |view, window, cx| {
                    window.focus(&view.focus_handle);
                    cx.activate(true);
                })
                .unwrap();
        });
    });

    remove_lock();
}
