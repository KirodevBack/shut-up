use super::wide;
use crate::platform::{app_name, Labels};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND, LUID};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDOK, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OKCANCEL, SW_HIDE,
};

#[link(name = "ntdll")]
extern "system" {
    fn NtSetSystemInformation(class: i32, information: *const c_void, length: u32) -> i32;
}

pub fn elevated_flag() -> &'static str {
    "--clear-cache"
}

fn message(window: HWND, text: &str, style: u32) -> i32 {
    let text = wide::null_terminated(text);
    let title = wide::null_terminated(app_name());
    unsafe { MessageBoxW(window, text.as_ptr(), title.as_ptr(), style) }
}

pub fn ask_and_clear(window: HWND, labels: &Labels) {
    if message(window, labels.clear_cache_warning, MB_OKCANCEL | MB_ICONWARNING) != IDOK {
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let exe = wide::null_terminated(&exe.display().to_string());
    let verb = wide::null_terminated("runas");
    let args = wide::null_terminated(elevated_flag());
    unsafe {
        ShellExecuteW(null_mut(), verb.as_ptr(), exe.as_ptr(), args.as_ptr(), null(), SW_HIDE);
    }
}

unsafe fn enable_privilege(name: &str) -> bool {
    let mut token: HANDLE = null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) == 0 {
        return false;
    }
    let name = wide::null_terminated(name);
    let mut luid: LUID = zeroed();
    let found = LookupPrivilegeValueW(null(), name.as_ptr(), &mut luid) != 0;
    let privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    let adjusted = found
        && AdjustTokenPrivileges(token, 0, &privileges, size_of::<TOKEN_PRIVILEGES>() as u32, null_mut(), null_mut()) != 0;
    CloseHandle(token);
    adjusted
}

unsafe fn memory_list_command(command: i32) -> bool {
    let memory_list_information = 80;
    let status = NtSetSystemInformation(
        memory_list_information,
        &command as *const i32 as *const c_void,
        size_of::<i32>() as u32,
    );
    status >= 0
}

pub fn clear_elevated() {
    let labels = Labels::default();
    let flush_modified = 3;
    let purge_standby = 4;
    let done = unsafe {
        enable_privilege("SeProfileSingleProcessPrivilege")
            && memory_list_command(flush_modified)
            && memory_list_command(purge_standby)
    };
    if done {
        message(null_mut(), labels.clear_cache_done, MB_ICONINFORMATION);
    } else {
        message(null_mut(), labels.clear_cache_failed, MB_ICONERROR);
    }
}
