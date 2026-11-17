use super::{process, wide};
use crate::app::App;
use crate::platform::{app_id, app_name, Labels};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    NOTIFY_ICON_MESSAGE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone, Copy)]
enum Command {
    Quit,
    ToggleAll,
    Toggle(usize),
}

impl Command {
    fn first_target_id() -> usize {
        100
    }

    fn id(self) -> usize {
        match self {
            Command::Quit => 1,
            Command::ToggleAll => 2,
            Command::Toggle(index) => Self::first_target_id() + index,
        }
    }

    fn from_id(id: usize) -> Option<Self> {
        match id {
            1 => Some(Command::Quit),
            2 => Some(Command::ToggleAll),
            id if id >= Self::first_target_id() => Some(Command::Toggle(id - Self::first_target_id())),
            _ => None,
        }
    }
}

struct TrayWindow {
    app: &'static App,
    labels: Labels,
    icon: HICON,
    icon_id: u32,
    callback_message: u32,
    refresh_timer: usize,
    taskbar_created: u32,
}

impl TrayWindow {
    unsafe fn new(app: &'static App) -> Self {
        let taskbar_created = wide::null_terminated("TaskbarCreated");
        Self {
            app,
            labels: Labels::default(),
            icon: load_icon(),
            icon_id: 1,
            callback_message: WM_APP + 1,
            refresh_timer: 1,
            taskbar_created: RegisterWindowMessageW(taskbar_created.as_ptr()),
        }
    }

    unsafe fn notify(&self, window: HWND, message: NOTIFY_ICON_MESSAGE) {
        let mut data: NOTIFYICONDATAW = zeroed();
        data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = window;
        data.uID = self.icon_id;
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = self.callback_message;
        data.hIcon = self.icon;
        wide::copy_into(&mut data.szTip, app_name());
        Shell_NotifyIconW(message, &data);
    }

    unsafe fn build_menu(&self) -> HMENU {
        let app = self.app;
        let targets = CreatePopupMenu();
        for (index, target) in app.targets().iter().enumerate() {
            let flags = MF_STRING | check_flag(app.is_enabled(index));
            append(targets, flags, Command::Toggle(index).id(), &target.name);
        }
        AppendMenuW(targets, MF_SEPARATOR, 0, null());
        let all = MF_STRING | check_flag(app.all_enabled());
        append(targets, all, Command::ToggleAll.id(), self.labels.toggle_all);

        let root = CreatePopupMenu();
        append(root, MF_POPUP, targets as usize, self.labels.submenu);
        AppendMenuW(root, MF_SEPARATOR, 0, null());
        append(root, MF_STRING, Command::Quit.id(), self.labels.quit);
        root
    }

    unsafe fn show_menu(&self, window: HWND) {
        let menu = self.build_menu();

        let mut cursor: POINT = zeroed();
        GetCursorPos(&mut cursor);
        SetForegroundWindow(window);
        let flags = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN;
        let chosen = TrackPopupMenu(menu, flags, cursor.x, cursor.y, 0, window, null());
        PostMessageW(window, WM_NULL, 0, 0);
        DestroyMenu(menu);

        match Command::from_id(chosen as usize) {
            Some(Command::Quit) => {
                DestroyWindow(window);
            }
            Some(Command::ToggleAll) => self.app.toggle_all(),
            Some(Command::Toggle(index)) => self.app.toggle(index),
            None => {}
        }
        process::trim_current();
    }

    unsafe fn handle(&self, window: HWND, message: u32, lparam: LPARAM) -> Option<LRESULT> {
        match message {
            m if m == self.callback_message => {
                let event = (lparam & 0xffff) as u32;
                if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                    self.show_menu(window);
                }
            }
            m if m != 0 && m == self.taskbar_created => self.notify(window, NIM_ADD),
            WM_TIMER => {
                self.app.enforce();
                process::trim_current();
            }
            WM_DESTROY => {
                KillTimer(window, self.refresh_timer);
                self.notify(window, NIM_DELETE);
                self.app.shutdown();
                PostQuitMessage(0);
            }
            _ => return None,
        }
        Some(0)
    }
}

impl Drop for TrayWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyIcon(self.icon);
        }
    }
}

unsafe fn load_icon() -> HICON {
    let size = GetSystemMetrics(SM_CXSMICON);
    let app_icon = std::ptr::without_provenance::<u16>(1);
    LoadImageW(GetModuleHandleW(null()), app_icon, IMAGE_ICON, size, size, LR_DEFAULTCOLOR)
}

unsafe fn append(menu: HMENU, flags: MENU_ITEM_FLAGS, id: usize, label: &str) {
    let text = wide::null_terminated(label);
    AppendMenuW(menu, flags, id, text.as_ptr());
}

fn check_flag(checked: bool) -> MENU_ITEM_FLAGS {
    if checked {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    }
}

unsafe fn tray_of<'a>(window: HWND) -> Option<&'a TrayWindow> {
    (GetWindowLongPtrW(window, GWLP_USERDATA) as *const TrayWindow).as_ref()
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = &*(lparam as *const CREATESTRUCTW);
            SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        WM_NCDESTROY => {
            let raw = SetWindowLongPtrW(window, GWLP_USERDATA, 0) as *mut TrayWindow;
            if !raw.is_null() {
                drop(Box::from_raw(raw));
            }
        }
        _ => {
            if let Some(result) = tray_of(window).and_then(|t| t.handle(window, message, lparam)) {
                return result;
            }
        }
    }
    DefWindowProcW(window, message, wparam, lparam)
}

unsafe fn create_window(tray: Box<TrayWindow>) -> Option<HWND> {
    let instance = GetModuleHandleW(null());
    let class_name = wide::null_terminated(app_id());

    let mut class: WNDCLASSW = zeroed();
    class.lpfnWndProc = Some(window_proc);
    class.hInstance = instance;
    class.lpszClassName = class_name.as_ptr();
    RegisterClassW(&class);

    let state = Box::into_raw(tray);
    let window = CreateWindowExW(
        0,
        class_name.as_ptr(),
        class_name.as_ptr(),
        0,
        0,
        0,
        0,
        0,
        null_mut(),
        null_mut(),
        instance,
        state as *const c_void,
    );
    if window.is_null() {
        drop(Box::from_raw(state));
        return None;
    }
    Some(window)
}

pub fn run(app: &'static App) {
    let interval = app.config().refresh_interval.as_millis().min(u32::MAX as u128) as u32;

    unsafe {
        let Some(window) = create_window(Box::new(TrayWindow::new(app))) else {
            return;
        };
        if let Some(tray) = tray_of(window) {
            tray.notify(window, NIM_ADD);
            SetTimer(window, tray.refresh_timer, interval, None);
        }
        process::trim_current();

        let mut message: MSG = zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
