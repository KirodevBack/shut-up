use crate::platform::{app_name, Labels};
use std::io::ErrorKind;
use std::process::{Command, ExitStatus};

fn run(program: &str, args: &[&str]) -> Option<std::io::Result<ExitStatus>> {
    match Command::new(program).args(args).status() {
        Err(err) if err.kind() == ErrorKind::NotFound => None,
        result => Some(result),
    }
}

pub fn confirm(labels: &Labels, text: &str) -> bool {
    let ok = format!("--ok-label={}", labels.continue_button);
    let cancel = format!("--cancel-label={}", labels.cancel_button);
    let zenity = ["--question", "--icon-name=dialog-warning", "--title", app_name(), &ok, &cancel, "--text", text];
    if let Some(result) = run("zenity", &zenity) {
        return result.is_ok_and(|status| status.success());
    }
    let kdialog = ["--title", app_name(), "--warningcontinuecancel", text];
    match run("kdialog", &kdialog) {
        Some(result) => result.is_ok_and(|status| status.success()),
        None => true,
    }
}

pub fn warn(text: &str) {
    let zenity = ["--warning", "--title", app_name(), "--text", text];
    if run("zenity", &zenity).is_some() {
        return;
    }
    if run("kdialog", &["--title", app_name(), "--sorry", text]).is_some() {
        return;
    }
    let _ = run("notify-send", &[app_name(), text]);
}

pub fn notify(text: &str) {
    let _ = run("notify-send", &[app_name(), text]);
}
