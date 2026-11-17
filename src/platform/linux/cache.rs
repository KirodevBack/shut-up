use crate::platform::{app_name, Labels};
use std::io::ErrorKind;
use std::process::Command;
use std::thread;

fn confirm(labels: &Labels) -> bool {
    let zenity = Command::new("zenity")
        .args(["--question", "--icon-name=dialog-warning", "--title", app_name(), "--text"])
        .arg(labels.clear_cache_warning)
        .status();
    match zenity {
        Ok(status) => return status.success(),
        Err(err) if err.kind() != ErrorKind::NotFound => return false,
        Err(_) => {}
    }

    let kdialog = Command::new("kdialog")
        .args(["--title", app_name(), "--warningcontinuecancel"])
        .arg(labels.clear_cache_warning)
        .status();
    match kdialog {
        Ok(status) => status.success(),
        Err(err) => err.kind() == ErrorKind::NotFound,
    }
}

pub fn ask_and_clear(labels: &'static Labels) {
    thread::spawn(move || {
        if !confirm(labels) {
            return;
        }
        let cleared = Command::new("pkexec")
            .args(["sh", "-c", "sync && echo 3 > /proc/sys/vm/drop_caches"])
            .status()
            .is_ok_and(|status| status.success());
        let text = if cleared { labels.clear_cache_done } else { labels.clear_cache_failed };
        let _ = Command::new("notify-send").args([app_name(), text]).status();
    });
}
