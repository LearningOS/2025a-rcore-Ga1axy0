//! Process management syscalls
use crate::config::PAGE_SIZE_BITS;
use crate::mm::{MapPermission, PTEFlags, PageTable, VirtAddr, translated_byte_buffer};
use crate::task::{current_user_token, get_syscall_count};
use crate::task::{
    change_program_brk, exit_current_and_run_next, mmap_current_task, munmap_current_task,
    suspend_current_and_run_next,
};
use crate::timer::get_time_us;
use core::mem::size_of;
use core::slice;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let time_us = get_time_us();
    let timeval = TimeVal {
        sec: time_us / 1_000_000,
        usec: time_us % 1_000_000,
    };
    let timeval_bytes = unsafe {
        slice::from_raw_parts(&timeval as *const TimeVal as *const u8, size_of::<TimeVal>())
    };
    let mut buffers = translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    let mut copied = 0usize;
    for buffer in buffers.iter_mut() {
        let len = buffer.len();
        buffer.copy_from_slice(&timeval_bytes[copied..copied + len]);
        copied += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
fn user_readable(ptr: *const u8, len: usize) -> bool {
    if len == 0 { return true; }
    let start = ptr as usize;
    let end = match start.checked_add(len) { Some(v) => v, None => return false };
    let pt = PageTable::from_token(current_user_token());
    let mut va = start;
    while va < end {
        let vpn = VirtAddr::from(va).floor();
        let pte = match pt.translate(vpn) { Some(p) => p, None => return false };
        let f = pte.flags();
        if !(f.contains(PTEFlags::V) && f.contains(PTEFlags::U) && f.contains(PTEFlags::R)) {
            return false;
        }
        va = ((va >> 12) + 1) << 12;
    }
    true
}

fn user_writable(ptr: *const u8, len: usize) -> bool {
    if len == 0 { return true; }
    let start = ptr as usize;
    let end = match start.checked_add(len) { Some(v) => v, None => return false };
    let pt = PageTable::from_token(current_user_token());
    let mut va = start;
    while va < end {
        let vpn = VirtAddr::from(va).floor();
        let pte = match pt.translate(vpn) { Some(p) => p, None => return false };
        let f = pte.flags();
        if !(f.contains(PTEFlags::V) && f.contains(PTEFlags::U) && f.contains(PTEFlags::W)) {
            return false;
        }
        va = ((va >> 12) + 1) << 12;
    }
    true
}

fn trace_fn_0(_id: usize) -> isize{
    let ptr = _id as *const u8;
    if !user_readable(ptr, 1) {
        return -1;
    }
    let buffers = translated_byte_buffer(current_user_token(), ptr, 1);
    buffers[0][0] as isize
}
fn trace_fn_1(_id: usize, _data: usize) -> isize{
    let ptr = _id as *const u8;
    if !user_writable(ptr, 1) {
        return -1;
    }
    let mut buffers = translated_byte_buffer(current_user_token(), ptr, 1);
    buffers[0][0] = _data as u8;
    0
}

fn trace_fn_2(_id: usize) -> isize{
    get_syscall_count(_id).unwrap_or(0) as isize
}

pub fn sys_trace(trace_request: usize, _id: usize, _data: usize) -> isize {
    match trace_request {
        0 => trace_fn_0(_id),
        1 => trace_fn_1(_id, _data),
        2 => trace_fn_2(_id),
        _ => -1,
    }
}



// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if _start & ((1 << PAGE_SIZE_BITS) - 1) != 0 {
        return -1;
    }
    if _port & !0x7 != 0 {
        return -1;
    }
    if _port & 0x7 == 0 {
        return -1;
    }
    let Some(end) = _start.checked_add(_len) else {
        return -1;
    };

    let mut perm = MapPermission::U;
    if _port & 0x1 != 0 {
        perm |= MapPermission::R;
    }
    if _port & 0x2 != 0 {
        perm |= MapPermission::W;
    }
    if _port & 0x4 != 0 {
        perm |= MapPermission::X;
    }

    if mmap_current_task(VirtAddr::from(_start), VirtAddr::from(end), perm) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start & ((1 << PAGE_SIZE_BITS) - 1) != 0 {
        return -1;
    }
    let Some(end) = _start.checked_add(_len) else {
        return -1;
    };
    if munmap_current_task(VirtAddr::from(_start), VirtAddr::from(end)) {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
