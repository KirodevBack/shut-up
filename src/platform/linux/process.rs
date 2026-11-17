use libc::{c_int, c_void, iovec, pid_t};
use std::fs;
use std::path::PathBuf;

struct Nice;

impl Nice {
    fn for_mode(efficient: bool) -> c_int {
        if efficient {
            19
        } else {
            0
        }
    }
}

struct IoPriority;

impl IoPriority {
    fn who_process() -> c_int {
        1
    }

    fn for_mode(efficient: bool) -> c_int {
        let class_shift = 13;
        let idle_class = 3;
        let default_class = 0;
        let class = if efficient { idle_class } else { default_class };
        class << class_shift
    }
}

fn parse_pid(name: &[u8]) -> Option<u32> {
    if name.is_empty() || !name.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(name).ok()?.parse().ok()
}

pub fn for_each(mut visit: impl FnMut(u32, &str)) {
    let Ok(entries) = fs::read_dir("/proc") else { return };
    let mut path = PathBuf::with_capacity(32);

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = parse_pid(name.as_encoded_bytes()) else { continue };

        path.clear();
        path.extend(["/proc".as_ref(), name.as_os_str(), "comm".as_ref()]);
        if let Ok(comm) = fs::read_to_string(&path) {
            visit(pid, comm.trim_end());
        }
    }
}

fn for_each_thread(pid: u32, mut visit: impl FnMut(pid_t)) {
    let Ok(entries) = fs::read_dir(format!("/proc/{pid}/task")) else { return };
    for entry in entries.flatten() {
        if let Some(tid) = parse_pid(entry.file_name().as_encoded_bytes()) {
            visit(tid as pid_t);
        }
    }
}

unsafe fn set_scheduler(tid: pid_t, efficient: bool) {
    let param = libc::sched_param { sched_priority: 0 };
    let policy = if efficient { libc::SCHED_IDLE } else { libc::SCHED_OTHER };
    libc::sched_setscheduler(tid, policy, &param);
    libc::setpriority(libc::PRIO_PROCESS, tid as libc::id_t, Nice::for_mode(efficient));
}

unsafe fn set_thread_priority(tid: pid_t, efficient: bool) {
    set_scheduler(tid, efficient);
    libc::syscall(
        libc::SYS_ioprio_set,
        IoPriority::who_process(),
        tid,
        IoPriority::for_mode(efficient),
    );
}

fn parse_range(line: &str) -> Option<iovec> {
    let kernel_mappings = ["[vsyscall]", "[vvar]", "[vdso]"];
    if kernel_mappings.iter().any(|m| line.ends_with(m)) {
        return None;
    }
    let (range, _) = line.split_once(' ')?;
    let (start, end) = range.split_once('-')?;
    let start = usize::from_str_radix(start, 16).ok()?;
    let end = usize::from_str_radix(end, 16).ok()?;
    Some(iovec {
        iov_base: start as *mut c_void,
        iov_len: end.saturating_sub(start),
    })
}

struct PidFd(c_int);

impl PidFd {
    fn open(pid: u32) -> Option<Self> {
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid as pid_t, 0) } as c_int;
        (fd >= 0).then_some(Self(fd))
    }

    fn page_out(&self, ranges: &[iovec]) {
        let madv_pageout: c_int = 21;
        if ranges.is_empty() {
            return;
        }
        unsafe {
            libc::syscall(
                libc::SYS_process_madvise,
                self.0,
                ranges.as_ptr(),
                ranges.len(),
                madv_pageout,
                0,
            );
        }
    }
}

impl Drop for PidFd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

fn trim(pid: u32) {
    let batch_size = 512;
    let Ok(maps) = fs::read_to_string(format!("/proc/{pid}/maps")) else { return };
    let Some(fd) = PidFd::open(pid) else { return };

    let ranges: Vec<iovec> = maps.lines().filter_map(parse_range).collect();
    for batch in ranges.chunks(batch_size) {
        fd.page_out(batch);
    }
}

pub fn set_efficiency(pid: u32, efficient: bool, trim_memory: bool) {
    for_each_thread(pid, |tid| unsafe { set_thread_priority(tid, efficient) });
    if trim_memory {
        trim(pid);
    }
}

pub fn make_current_efficient() {
    unsafe { set_scheduler(0, true) }
}
