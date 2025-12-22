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
use applications::{load_from_cache, save_to_cache, scan_applications};
use calculator::{is_calculator_query, Calculator};
use fuzzy::FuzzyMatcher;
use process::{get_running_processes, is_process_query, kill_process, ProcessInfo};
use state::ApplicationInfo;

actions!(launcher, [Quit, Backspace, Up, Down, Launch]);

enum SearchResult {
    Application(usize),
    Calculation(String),
    Process(ProcessInfo),
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
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications: Vec::new(),
            query: "".into(),
            selected_index: 0,
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: None,
            search_results: Vec::new(),
            scroll_offset: 0,
        };

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

        if is_process_query(&query_str) {
            let processes = get_running_processes();
            if query_str == "ps" {
                for process in processes.into_iter().take(20) {
                    self.search_results.push(SearchResult::Process(process));
                }
            } else if query_str.starts_with("kill ") {
                let search_term = query_str.strip_prefix("kill ").unwrap_or("").to_lowercase();
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
                    cx.quit();
                }
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
                match event.keystroke.key.as_str() {
                    " " => {
                        let mut query = this.query.to_string();
                        query.push(' ');
                        this.query = query.into();
                        this.update_search_results();
                        cx.notify();
                    }
                    key if key.len() == 1 => {
                        if let Some(key_char) = key.chars().next() {
                            if key_char.is_alphanumeric() || "+-*/()^.=".contains(key_char) {
                                let mut query = this.query.to_string();
                                query.push(key_char);
                                this.query = query.into();
                                this.update_search_results();
                                cx.notify();
                            }
                        }
                    }
                    _ => {}
                }
            }))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::launch))
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
                            .text_color(if query_text.is_empty() {
                                rgba(0xd8dee966)
                            } else {
                                rgb(0xeceff4)
                            })
                            .child(if query_text.is_empty() {
                                "Search for apps and commands".to_string()
                            } else {
                                query_text.clone()
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
