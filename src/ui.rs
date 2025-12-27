use gtk::{Application, ApplicationWindow, Box, Orientation, Entry, ListView, ScrolledWindow, SignalListItemFactory, Label, prelude::*, gio};
use gtk4_layer_shell::{Layer, Edge, KeyboardMode, LayerShell};
use std::env;

pub struct LauncherUi {
    pub window: ApplicationWindow,
    pub container: Box,
    pub search_entry: Entry,
    pub list_view: ListView,
    pub scrolled_window: ScrolledWindow,
}

pub fn build_ui(app: &Application) -> LauncherUi {
    let window = ApplicationWindow::builder()
        .application(app)
        .css_classes(vec!["launcher"])
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    
    // Ne pas définir le mode clavier exclusif si GTK_DEBUG=interactive
    if env::var("GTK_DEBUG").unwrap_or_default() != "interactive" {
        window.set_keyboard_mode(KeyboardMode::Exclusive);
    }

    window.set_size_request(500, 300);
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_margin(Edge::Top, 100);

    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(vec!["container"])
        .build();

    let search_entry = Entry::builder()
        .placeholder_text("Search applications...")
        .css_classes(vec!["search-entry"])
        .can_focus(true)
        .build();

    search_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("system-search-symbolic"));

    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let hbox = Box::new(Orientation::Horizontal, 5);
        let icon = gtk::Image::new();
        icon.set_css_classes(&["icon"]);
        // Optimization: Set fixed icon size to avoid resize calculations during scroll
        icon.set_pixel_size(32);
        
        let label = Label::new(None);
        hbox.append(&icon);
        hbox.append(&label);
        item.set_child(Some(&hbox));
    });

    factory.connect_bind(move |_factory, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let hbox = item.child().and_downcast::<Box>().unwrap();
        let icon = hbox.first_child().and_downcast::<gtk::Image>().unwrap();
        let label = hbox.last_child().and_downcast::<Label>().unwrap();
        let app_info = item.item().and_downcast::<gio::AppInfo>().unwrap();

        if let Some(gicon) = app_info.icon() {
            icon.set_from_gicon(&gicon);
        } else {
            icon.set_icon_name(Some("application-x-executable"));
        }
        label.set_text(&app_info.name());
    });

    let list_view = ListView::new(None::<gtk::SingleSelection>, Some(factory));

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .min_content_height(300)
        .child(&list_view)
        .build();

    container.append(&search_entry);
    container.append(&scrolled_window);

    window.set_child(Some(&container));

    LauncherUi {
        window,
        container,
        search_entry,
        list_view,
        scrolled_window,
    }
}