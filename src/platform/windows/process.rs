use super::wide;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, ProcessMemoryPriority, ProcessPowerThrottling,
    SetPriorityClass, SetProcessInformation, SetProcessWorkingSetSize, IDLE_PRIORITY_CLASS,
    MEMORY_PRIORITY_INFORMATION, MEMORY_PRIORITY_NORMAL, MEMORY_PRIORITY_VERY_LOW,
    NORMAL_PRIORITY_CLASS, PROCESS_INFORMATION_CLASS, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
    PROCESS_SET_QUOTA,
};

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn new(raw: HANDLE) -> Option<Self> {
        (!raw.is_null() && raw != INVALID_HANDLE_VALUE).then_some(Self(raw))
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn for_each(mut visit: impl FnMut(u32, &str)) {
    unsafe {
        let Some(snapshot) = OwnedHandle::new(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)) else {
            return;
        };
        let mut entry: PROCESSENTRY32W = zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut name = [0u8; 64];

        let mut more = Process32FirstW(snapshot.0, &mut entry) != 0;
        while more {
            if let Some(exe) = wide::to_ascii(&entry.szExeFile, &mut name) {
                visit(entry.th32ProcessID, exe);
            }
            more = Process32NextW(snapshot.0, &mut entry) != 0;
        }
    }
}

unsafe fn set_info<T>(process: HANDLE, class: PROCESS_INFORMATION_CLASS, value: &T) {
    SetProcessInformation(
        process,
        class,
        value as *const T as *const c_void,
        size_of::<T>() as u32,
    );
}

unsafe fn set_qos(process: HANDLE, efficient: bool) {
    let qos = PROCESS_POWER_THROTTLING_EXECUTION_SPEED | PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION;
    let mask = if efficient { qos } else { 0 };
    let throttling = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: mask,
        StateMask: mask,
    };
    set_info(process, ProcessPowerThrottling, &throttling);

    let memory_priority = if efficient { MEMORY_PRIORITY_VERY_LOW } else { MEMORY_PRIORITY_NORMAL };
    let memory = MEMORY_PRIORITY_INFORMATION { MemoryPriority: memory_priority };
    set_info(process, ProcessMemoryPriority, &memory);
}

unsafe fn trim(process: HANDLE) {
    SetProcessWorkingSetSize(process, usize::MAX, usize::MAX);
}

pub fn set_efficiency(pid: u32, efficient: bool, trim_memory: bool) {
    let access = PROCESS_SET_INFORMATION | PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION;
    unsafe {
        let Some(process) = OwnedHandle::new(OpenProcess(access, 0, pid)) else {
            return;
        };
        let priority = if efficient { IDLE_PRIORITY_CLASS } else { NORMAL_PRIORITY_CLASS };
        SetPriorityClass(process.0, priority);
        set_qos(process.0, efficient);
        if trim_memory {
            trim(process.0);
        }
    }
}

pub fn make_current_efficient() {
    unsafe {
        let me = GetCurrentProcess();
        SetPriorityClass(me, IDLE_PRIORITY_CLASS);
        set_qos(me, true);
    }
}

pub fn trim_current() {
    unsafe { trim(GetCurrentProcess()) }
}
