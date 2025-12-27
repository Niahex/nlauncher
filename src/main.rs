use nlauncher::launcher::Launcher;
use nlauncher::styles::load_css;
use gtk::{Application, prelude::*, glib, gio};
use socket2::{Socket, Domain, Type};
use std::io::Read;
use std::sync::mpsc;
use std::env;
use nlauncher::cache;
use nlauncher::applications::Applications;
use std::thread;
use std::collections::HashSet;

const APP_ID: &str = "github.niahex.nlauncher";
const LOCK_PATH: &str = "/tmp/nlauncher.sock";

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--refresh".to_string()) {
        if let Err(e) = cache::clear_cache() {
            eprintln!("Failed to clear cache: {e}");
        }
    }

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
        let app_list_store = Applications::get_cached_applications();
        let launcher = Launcher::new(app, app_list_store.clone());
        launcher.init();
        launcher.show();

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let app_ids = Applications::scan_for_applications();
            tx.send(app_ids).unwrap();
        });

        glib::idle_add_local(move || {
            if let Ok(new_app_ids) = rx.try_recv() {
                let cached_app_ids: HashSet<String> = app_list_store
                    .iter::<gio::AppInfo>()
                    .filter_map(|app_info| app_info.ok())
                    .filter_map(|app_info| app_info.id().map(|s| s.to_string()))
                    .collect();

                // Add new apps
                for id in new_app_ids.difference(&cached_app_ids) {
                    if let Some(desktop_app_info) = gio::DesktopAppInfo::new(id) {
                        let app_info = desktop_app_info.upcast::<gio::AppInfo>();
                        app_list_store.append(&app_info);
                    }
                }

                // Remove old apps
                let mut to_remove = Vec::new();
                for (i, app_info) in app_list_store.iter::<gio::AppInfo>().enumerate() {
                    if let Ok(app_info) = app_info {
                        if let Some(id) = app_info.id() {
                            if !new_app_ids.contains(id.as_str()) {
                                to_remove.push(i as u32);
                            }
                        }
                    }
                }
                to_remove.reverse(); // Remove from the end to avoid index shifting
                for i in to_remove {
                    app_list_store.remove(i);
                }
                
                // Save to cache (sorted for stability)
                let mut sorted_ids: Vec<_> = new_app_ids.into_iter().collect();
                sorted_ids.sort();
                
                if let Err(e) = cache::save_to_cache(&sorted_ids) {
                    eprintln!("Failed to save app cache: {e}");
                }

                return glib::ControlFlow::Break;
            }
            
            glib::ControlFlow::Continue
        });
    });

    app.run();

    // Nettoyer le fichier de socket en quittant.
    let _ = std::fs::remove_file(LOCK_PATH);
}
