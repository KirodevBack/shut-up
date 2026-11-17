pub mod process;
mod tray;

use crate::app::App;
use crate::platform::app_id;
use std::path::PathBuf;
use std::thread;

pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|dir| dir.join(app_id()))
}

pub fn run(app: &'static App) {
    process::make_current_efficient();
    app.enforce();

    let _tray = match tray::spawn(app) {
        Ok(handle) => handle,
        Err(err) => {
            eprintln!("{}: impossible d'afficher l'icone ({err})", app_id());
            std::process::exit(1);
        }
    };

    loop {
        thread::sleep(app.config().refresh_interval);
        app.enforce();
    }
}
