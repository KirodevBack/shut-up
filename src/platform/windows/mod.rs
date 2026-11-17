mod cache;
pub mod process;
mod tray;
mod wide;

use crate::app::App;
use crate::platform::app_id;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

pub fn config_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join(app_id()))
}

struct SingleInstance(HANDLE);

impl SingleInstance {
    fn acquire() -> Option<Self> {
        let name = wide::null_terminated(&format!("Local\\{}-tray", app_id()));
        unsafe {
            let handle = CreateMutexW(std::ptr::null(), 1, name.as_ptr());
            if handle.is_null() {
                return None;
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return None;
            }
            Some(Self(handle))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn run(app: &'static App) {
    if std::env::args().any(|arg| arg == cache::elevated_flag()) {
        cache::clear_elevated();
        return;
    }
    let Some(_guard) = SingleInstance::acquire() else {
        return;
    };
    process::make_current_efficient();
    app.enforce();
    tray::run(app);
}
