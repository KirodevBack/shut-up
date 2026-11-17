mod cache;
pub mod process;
mod tray;

use crate::app::App;
use crate::platform::app_id;
use std::fs;
use std::path::PathBuf;
use std::thread;

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn config_dir() -> Option<PathBuf> {
    home().map(|h| h.join("Library").join("Application Support").join(app_id()))
}

fn bundle_id() -> String {
    format!("com.metixw.{}", app_id())
}

fn register_login_item() {
    let Ok(exe) = std::env::current_exe() else { return };
    let exe = exe.display().to_string();
    if !exe.contains(".app/Contents/MacOS/") {
        return;
    }
    let Some(agents) = home().map(|h| h.join("Library").join("LaunchAgents")) else {
        return;
    };

    let label = bundle_id();
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#
    );

    let path = agents.join(format!("{label}.plist"));
    if fs::read_to_string(&path).ok().as_deref() != Some(plist.as_str()) {
        let _ = fs::create_dir_all(&agents);
        let _ = fs::write(path, plist);
    }
}

pub fn run(app: &'static App) {
    process::make_current_efficient();
    register_login_item();
    app.enforce();

    thread::spawn(move || loop {
        thread::sleep(app.config().refresh_interval);
        app.enforce();
    });

    tray::run(app);
}
