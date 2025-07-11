use nlauncher::launcher::Launcher;
use nlauncher::styles::load_css;
use gtk::{Application, prelude::*, glib};
use socket2::{Socket, Domain, Type};
use std::io::Read;
use std::sync::mpsc;

const APP_ID: &str = "github.niahex.nwidgets.launcher";
const LOCK_PATH: &str = "/tmp/nlauncher.sock";

fn main() {
    // Essayer de se connecter à une instance existante.
    let client_socket = Socket::new(Domain::UNIX, Type::STREAM, None).unwrap();
    let addr = socket2::SockAddr::unix(LOCK_PATH).unwrap();

    if client_socket.connect(&addr).is_ok() {
        // Une instance est déjà en cours, on lui envoie un message pour quitter.
        let _ = client_socket.send(b"quit");
        println!("nlauncher is already running. Sending quit signal.");
        return; // La nouvelle instance se ferme.
    }

    // Aucune instance en cours, on devient le serveur.
    let _ = std::fs::remove_file(LOCK_PATH);
    let server_socket = Socket::new(Domain::UNIX, Type::STREAM, None).unwrap();
    server_socket.bind(&addr).expect("Failed to bind to lock socket");
    server_socket.listen(1).expect("Failed to listen on lock socket");

    let app = Application::builder().application_id(APP_ID).build();

    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = server_socket.accept() {
            let mut buffer = [0; 4];
            if stream.read_exact(&mut buffer).is_ok() && &buffer == b"quit" {
                tx.send(()).expect("Failed to send quit signal");
            }
        }
    });

    let app_clone = app.clone();
    glib::idle_add_local(move || {
        if rx.try_recv().is_ok() {
            app_clone.quit();
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(|app| {
        let launcher = Launcher::new(app);
        launcher.init();
        launcher.show();
    });

    app.run();

    // Nettoyer le fichier de socket en quittant.
    let _ = std::fs::remove_file(LOCK_PATH);
}
