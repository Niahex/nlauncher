use nlauncher::launcher::Launcher;
use nlauncher::styles::load_css;
use gtk::{Application, prelude::*};

const APP_ID: &str = "github.niahex.nwidgets.launcher";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(|app| {
        let launcher = Launcher::new(app);
        launcher.init();
        launcher.show();
    });

    app.run();
}
