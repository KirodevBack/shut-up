use libc::{c_int, c_void, pid_t};

fn nice_for(efficient: bool) -> c_int {
    if efficient {
        20
    } else {
        0
    }
}

fn darwin_state_for(efficient: bool) -> c_int {
    if efficient {
        libc::PRIO_DARWIN_BG
    } else {
        0
    }
}

fn list_pids() -> Vec<pid_t> {
    unsafe {
        let needed = libc::proc_listallpids(std::ptr::null_mut(), 0);
        if needed <= 0 {
            return Vec::new();
        }
        let room = needed as usize + needed as usize / 4;
        let mut pids: Vec<pid_t> = vec![0; room];
        let bytes = (room * size_of::<pid_t>()) as c_int;
        let count = libc::proc_listallpids(pids.as_mut_ptr() as *mut c_void, bytes);
        pids.truncate(count.max(0) as usize);
        pids
    }
}

pub fn for_each(mut visit: impl FnMut(u32, &str)) {
    let mut name = [0u8; 64];
    for pid in list_pids() {
        if pid <= 0 {
            continue;
        }
        let len = unsafe { libc::proc_name(pid, name.as_mut_ptr() as *mut c_void, name.len() as u32) };
        if len <= 0 {
            continue;
        }
        if let Ok(process) = std::str::from_utf8(&name[..len as usize]) {
            visit(pid as u32, process);
        }
    }
}

pub fn set_efficiency(pid: u32, efficient: bool, _trim_memory: bool) {
    unsafe {
        libc::setpriority(libc::PRIO_DARWIN_PROCESS, pid as libc::id_t, darwin_state_for(efficient));
        libc::setpriority(libc::PRIO_PROCESS, pid as libc::id_t, nice_for(efficient));
    }
}

pub fn make_current_efficient() {
    set_efficiency(std::process::id(), true, false);
}
