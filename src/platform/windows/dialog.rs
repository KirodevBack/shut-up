use super::wide;
use crate::platform::app_name;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE};

pub fn show(window: HWND, text: &str, style: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    let text = wide::null_terminated(text);
    let title = wide::null_terminated(app_name());
    unsafe { MessageBoxW(window, text.as_ptr(), title.as_ptr(), style) }
}
