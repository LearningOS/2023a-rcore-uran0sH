//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translate_mut_ptr, MapPermission, VirtAddr},
    task::{
        alloc_new_frames, change_program_brk, check_all_allocated, check_allocated,
        current_user_token, dealloc_frames, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, TASK_MANAGER,
    },
    timer::{get_time_ms, get_time_us},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
    let us = get_time_us();
    let ts_ptr = translate_mut_ptr(current_user_token(), ts);
    if ts_ptr.is_none() {
        return -1;
    }
    let ts_ptr = ts_ptr.unwrap();
    unsafe {
        *ts_ptr = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let ti_ptr = translate_mut_ptr(current_user_token(), ti);
    if ti_ptr.is_none() {
        return -1;
    }
    let ti_ptr = ti_ptr.unwrap();
    unsafe {
        (*ti_ptr).status = TaskStatus::Running;
        (*ti_ptr).time = get_time_ms();
        (*ti_ptr).syscall_times = TASK_MANAGER.get_current_syscall_times();
        return 0;
    }
}

const MAX_ALLOC_MEMORY: usize = 1024 * 1024 * 1024;

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    // check port
    if port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }
    // check len
    if len > MAX_ALLOC_MEMORY {
        return -1;
    }
    let virt_addr_start: VirtAddr = start.into();
    let virt_addr_end: VirtAddr = (start + len).into();
    // check addr align
    if !virt_addr_start.aligned() {
        return -1;
    }
    // check if allocated
    if check_allocated(virt_addr_start, virt_addr_end) {
        return -1;
    }
    // allocate
    let per = MapPermission::from_bits(((port << 1) | 16) as u8).unwrap();
    alloc_new_frames(virt_addr_start, virt_addr_end, per);
    return 0;
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let virt_addr_start: VirtAddr = start.into();
    let virt_addr_end: VirtAddr = (start + len).into();
    if !virt_addr_start.aligned() {
        return -1;
    }
    // check
    if !check_all_allocated(virt_addr_start, virt_addr_end) {
        return -1;
    }
    dealloc_frames(virt_addr_start, virt_addr_end);
    0
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
