use gtk::gdk;

const CSS: &str = r#"
.launcher {
    background-color: rgba(0, 0, 0, 0.8);
    border-radius: 10px;
}

.container {
    padding: 10px;
}

.search-entry {
    margin-bottom: 10px;
    padding: 0 5px;
}

.icon {
    margin-right: 10px;
}
"#;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
