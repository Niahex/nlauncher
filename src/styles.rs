use gtk::gdk;

const CSS: &str = r#"
* {
    all: unset;
}

// .launcher {
//     margin 20px;
//     border-radius: 10px;
// }

.container {
    padding: 10px;
    margin 20px;
    border: 1px solid;
    border-radius: 10px;
    border-color: #8fbcbb;
    background-color: #2e3440;
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
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}
