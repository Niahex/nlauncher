use gpui::{
    actions, div, img, layer_shell::*, point, prelude::*, px, rgb, App, Application, Bounds,
    Context, FocusHandle, KeyBinding, KeyDownEvent, Render, SharedString, Size, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};
use std::process::Command;

mod applications;
mod calculator;
mod fuzzy;
mod process;
mod state;
use applications::load_applications;
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
    calculator: Calculator,
    search_results: Vec<SearchResult>,
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {
        let applications = load_applications();
        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications,
            query: "".into(),
            selected_index: 0,
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: Calculator::new(),
            search_results: Vec::new(),
            scroll_state: ScrollState::new(px(40.0)), // hauteur d'un item
        };
        launcher.update_search_results();
        launcher
    }

    fn update_search_results(&mut self) {
        let query_str = self.query.to_string();
        self.search_results.clear();

        // Vérifier si c'est une requête de processus
        if is_process_query(&query_str) {
            let processes = get_running_processes();
            if query_str == "ps" {
                // Afficher tous les processus
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
        }
        // Vérifier si c'est une requête de calcul
        else if is_calculator_query(&query_str) {
            if let Some(result) = self.calculator.evaluate(&query_str) {
                self.search_results.push(SearchResult::Calculation(result));
            }
        }
        // Recherche fuzzy dans les applications
        else {
            let app_indices = self.fuzzy_matcher.search(&query_str, &self.applications);
            for index in app_indices.into_iter().take(50) {
                self.search_results.push(SearchResult::Application(index));
            }
        }

        // Mettre à jour le scroll state
        self.scroll_state
            .update_item_count(self.search_results.len());

        // Réinitialiser la sélection
        self.selected_index = 0;
        self.scroll_state.scroll_to_reveal_item(0, 10);
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
            self.scroll_state
                .scroll_to_reveal_item(self.selected_index, 10);
            cx.notify();
        }
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if !self.search_results.is_empty() && self.selected_index + 1 < self.search_results.len() {
            self.selected_index += 1;
            self.scroll_state
                .scroll_to_reveal_item(self.selected_index, 10);
            cx.notify();
        }
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(result) = self.search_results.get(self.selected_index) {
            match result {
                SearchResult::Application(app_index) => {
                    if let Some(app) = self.applications.get(*app_index) {
                        let exec = app.exec.clone();

                        std::thread::spawn(move || {
                            let mut cmd = Command::new("sh");
                            cmd.arg("-c")
                                .arg(&exec)
                                .env_clear()
                                .env("PATH", std::env::var("PATH").unwrap_or_default())
                                .env("HOME", std::env::var("HOME").unwrap_or_default())
                                .env("USER", std::env::var("USER").unwrap_or_default())
                                .env(
                                    "XDG_RUNTIME_DIR",
                                    std::env::var("XDG_RUNTIME_DIR").unwrap_or_default(),
                                )
                                .env(
                                    "WAYLAND_DISPLAY",
                                    std::env::var("WAYLAND_DISPLAY").unwrap_or_default(),
                                )
                                .env("DISPLAY", std::env::var("DISPLAY").unwrap_or_default())
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null());

                            if let Err(err) = cmd.spawn() {
                                eprintln!("Failed to launch {exec}: {err}");
                            }
                        });

                        cx.quit();
                    }
                }
                SearchResult::Calculation(result) => {
                    // Copier le résultat dans le presse-papiers
                    if let Err(e) = std::process::Command::new("wl-copy").arg(result).output() {
                        eprintln!("Failed to copy to clipboard: {e}");
                    }
                    cx.quit();
                }
                SearchResult::Process(process) => {
                    let pid = process.pid;
                    std::thread::spawn(move || {
                        if let Err(e) = kill_process(pid) {
                            eprintln!("Failed to kill process: {e}");
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
            .bg(rgb(0x2e3440))
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
                    .bg(rgb(0x2e3440))
                    .border_1()
                    .border_color(rgb(0x4c566a))
                    .rounded_lg()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p_4()
                    .opacity(0.95)
                    .hover(|style| style.opacity(1.0))
                    .child(
                        div()
                            .p_2()
                            .bg(rgb(0x3b4252))
                            .rounded_md()
                            .text_color(rgb(0xeceff4))
                            .child(format!("Search: {query_text}")),
                    )
                    .child(div().flex().flex_col().mt_2().children({
                        let visible_range = self.scroll_state.visible_range();

                        self.search_results
                            .iter()
                            .enumerate()
                            .skip(visible_range.start)
                            .take(visible_range.len())
                            .map(|(original_index, result)| {
                                let mut item = div()
                                    .flex()
                                    .items_center()
                                    .p_2()
                                    .text_color(rgb(0xeceff4))
                                    .rounded_md()
                                    .hover(|style| style.bg(rgb(0x434c5e)));

                                if original_index == selected_index {
                                    item = item.bg(rgb(0x88c0d0)).text_color(rgb(0x2e3440));
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

fn main() {
    env_logger::init();

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
}
