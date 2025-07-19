use gtk::gdk;

const CSS: &str = r#"
* {
    all: unset;
}

.container {
    padding: 10px;
    margin 20px;
    border: 1px solid;
    border-radius: 10px;
    border-color: #8fbcbb;
    background-color: #2e3440;
}

.search-entry {
    -gtk-icon-size: 20px;
    background-color: #3b4252;
    border-radius: 5px;
    padding: 5px;
    margin-top: 10px;
}

.search-entry text {
    padding-left: 5px;
}

.view {
    -gtk-icon-size: 25px;
    margin-top: 10px;
    margin-bottom: 10px;
}

.view row {
    margin-top: 5px;
    margin-bottom: 5px;
    padding: 5px;
}

.view row:selected {
    background-color: #8fbcbb;
    color: #2e3440;
    border-radius: 5px;
}

"#;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}
