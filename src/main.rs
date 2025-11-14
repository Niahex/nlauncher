use gpui::{
    actions, App, Application, Bounds, Context, FocusHandle, KeyBinding, KeyDownEvent, Render,
    SharedString, Size, Window, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, div, img, layer_shell::*, point, prelude::*, px, rgb,
};
use std::process::Command;

mod applications;
mod state;
use applications::load_applications;
use state::ApplicationInfo;

actions!(launcher, [Quit, Backspace, Up, Down, Launch]);

struct Launcher {
    focus_handle: FocusHandle,
    applications: Vec<ApplicationInfo>,
    query: SharedString,
    selected_index: usize,
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {
        let applications = load_applications();
        Self {
            focus_handle: cx.focus_handle(),
            applications,
            query: "".into(),
            selected_index: 0,
        }
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let mut query = self.query.to_string();
        if !query.is_empty() {
            query.pop();
            self.query = query.into();
            self.selected_index = 0;
            cx.notify();
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        let filtered_count = self.filtered_apps().len();
        if filtered_count > 0 && self.selected_index > 0 {
            self.selected_index -= 1;
            cx.notify();
        }
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        let filtered_count = self.filtered_apps().len();
        if filtered_count > 0 && self.selected_index + 1 < filtered_count {
            self.selected_index += 1;
            cx.notify();
        }
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        let filtered_apps = self.filtered_apps();
        if let Some(app) = filtered_apps.get(self.selected_index) {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(&app.exec);
            if let Err(err) = cmd.spawn() {
                log::error!("Failed to launch application: {}", err);
            } else {
                cx.quit();
            }
        }
    }

    fn filtered_apps(&self) -> Vec<ApplicationInfo> {
        if self.query.is_empty() {
            self.applications.clone()
        } else {
            self.applications
                .iter()
                .filter(|app| {
                    app.name
                        .to_lowercase()
                        .contains(&self.query.to_lowercase())
                })
                .cloned()
                .collect()
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filtered_apps = self.filtered_apps();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(rgb(0x2e3440))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if let Some(key_char) = event.keystroke.key.chars().next() {
                    if key_char.is_alphanumeric() || key_char == ' ' {
                        let mut query = this.query.to_string();
                        query.push(key_char);
                        this.query = query.into();
                        this.selected_index = 0;
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::launch))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(
                        div()
                            .p_2()
                            .bg(rgb(0x3b4252))
                            .rounded_md()
                            .text_color(rgb(0xeceff4))
                            .child(format!("Search: {}", self.query))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .mt_2()
                            .children({
                                let filtered_apps = filtered_apps.clone();
                                let start_index = if self.selected_index >= 10 { self.selected_index - 9 } else { 0 };
                                
                                filtered_apps
                                    .into_iter()
                                    .enumerate()
                                    .skip(start_index)
                                    .take(10)
                                    .map(|(i, app)| {
                                        let mut item = div()
                                            .flex()
                                            .items_center()
                                            .p_2()
                                            .text_color(rgb(0xeceff4));
                                        
                                        if i == self.selected_index {
                                            item = item.bg(rgb(0x88c0d0));
                                        }
                                        
                                        item.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    if let Some(icon_path) = &app.icon_path {
                                                        div()
                                                            .size_6()
                                                            .child(img(std::path::PathBuf::from(icon_path)).size_6())
                                                    } else {
                                                        div()
                                                            .size_6()
                                                            .bg(rgb(0x5e81ac))
                                                            .rounded_sm()
                                                    }
                                                )
                                                .child(app.name.clone())
                                        )
                                    })
                            })
                    )
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

        let window = cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(600.), px(400.)),
                })),
                app_id: Some("nlauncher".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nlauncher".to_string(),
                    anchor: Anchor::BOTTOM,
                    margin: Some((px(0.), px(0.), px(50.), px(0.))),
                    keyboard_interactivity: KeyboardInteractivity::Exclusive,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Launcher::new),
        ).unwrap();

        window.update(cx, |view, window, cx| {
            window.focus(&view.focus_handle);
            cx.activate(true);
        }).unwrap();
    });
}
