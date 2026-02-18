//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};
use crate::task::get_syscall_count;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
fn trace_fn_0(_id: usize) -> isize{
    unsafe { *(_id as *const u8) as isize }
}
fn trace_fn_1(_id: usize, _data: usize) -> isize{
    unsafe{
        let ptr = _id as *mut u8;
        *ptr = _data as u8;
    }
    return 0;
}
fn trace_fn_2(_id: usize) -> isize{
    match get_syscall_count(_id) {
        Some(count) => count as isize,
        None => -1,
    }
}

pub fn sys_trace(trace_request: usize, _id: usize, _data: usize) -> isize {
    match trace_request {
        0 => trace_fn_0(_id),
        1 => trace_fn_1(_id, _data),
        2 => trace_fn_2(_id),
        _ => -1,
    }
}

