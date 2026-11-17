use super::dialog;
use crate::platform::Labels;
use std::process::Command;
use std::thread;

pub fn ask_and_clear(labels: &'static Labels) {
    thread::spawn(move || {
        if !dialog::confirm(labels, labels.clear_cache_warning) {
            return;
        }
        let cleared = Command::new("pkexec")
            .args(["sh", "-c", "sync && echo 3 > /proc/sys/vm/drop_caches"])
            .status()
            .is_ok_and(|status| status.success());
        dialog::notify(if cleared { labels.clear_cache_done } else { labels.clear_cache_failed });
    });
}
