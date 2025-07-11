use nlauncher::launcher::Launcher;
use nlauncher::styles::load_css;
use gtk::{Application, prelude::*};
use socket2::{Socket, Domain, Type};
use std::net::Shutdown;

const APP_ID: &str = "github.niahex.nwidgets.launcher";
const LOCK_PATH: &str = "/tmp/nlauncher.sock";

// Tente d'acquérir un verrou pour s'assurer qu'une seule instance est en cours.
// Retourne le socket de verrouillage en cas de succès, ou None si une autre instance est déjà en cours.
fn try_acquire_lock() -> Option<Socket> {
    let socket = Socket::new(Domain::UNIX, Type::STREAM, None).ok()?;
    let addr = socket2::SockAddr::unix(LOCK_PATH).ok()?;

    match socket.bind(&addr) {
        Ok(_) => {
            // Nous avons réussi à lier, nous sommes la première instance.
            Some(socket)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            // L'adresse est déjà utilisée, une autre instance est probablement en cours.
            // On essaie de se connecter pour être sûr.
            if socket.connect(&addr).is_ok() {
                // Connexion réussie -> une autre instance est bien en cours.
                println!("nlauncher is already running. Exiting.");
                None
            } else {
                // Connexion échouée -> le fichier de socket est probablement un reste d'un crash.
                // On le supprime et on réessaie de lier.
                let _ = std::fs::remove_file(LOCK_PATH);
                socket.bind(&addr).ok().map(|_| socket)
            }
        }
        Err(_) => {
            // Une autre erreur inattendue.
            None
        }
    }
}

fn main() {
    // Tenter d'acquérir le verrou. Si on ne peut pas, on quitte.
    let _lock_socket = match try_acquire_lock() {
        Some(socket) => socket,
        None => return,
    };

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

    // Le verrou est libéré lorsque _lock_socket est détruit à la fin de main.
    // On s'assure aussi de supprimer le fichier de socket.
    let _ = std::fs::remove_file(LOCK_PATH);
}
