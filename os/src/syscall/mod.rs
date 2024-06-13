//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// taskinfo syscall
const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;

use fs::*;
use lazy_static::lazy_static;
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use process::*;
use crate::config::{
        MAX_APP_NUM,
        MAX_SYSCALL_NUM};
use crate::task::TASK_MANAGER;
/// handle syscall exception with `syscall_id` and other arguments
#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // println!("{}",current_task);
    TOTAL_TASKS.add_syscall_times( syscall_id);
    match syscall_id {
        SYSCALL_WRITE => {

            sys_write(args[0], args[1] as *const u8, args[2])
        },
        SYSCALL_EXIT => {sys_exit(args[0] as i32)
        },
        SYSCALL_YIELD => {sys_yield()},
        SYSCALL_GET_TIME => {sys_get_time(args[0] as *mut TimeVal, args[1])},
        SYSCALL_TASK_INFO => {sys_task_info(args[0] as *mut TaskInfo)},
        _ => {panic!("Unsupported syscall_id: {}", syscall_id)},
    }
}
lazy_static! {
    /// 测试
    pub static ref TOTAL_TASKS:TotalTasks = unsafe{
    TotalTasks{
            inner:UPSafeCell::new([
                TaskStatBlock{
                    call_time:[0;MAX_SYSCALL_NUM],
                    start_time:0
                };
                MAX_APP_NUM])
        }
    };
}

/// test
pub struct TotalTasks{
    /// inner
    pub inner:UPSafeCell<[TaskStatBlock;MAX_APP_NUM]>
}

/// task的系统调用和开始时间统计
#[derive(Copy,Clone)]
pub struct TaskStatBlock{
    call_time:[u32;MAX_SYSCALL_NUM],
    start_time:usize
}

impl TotalTasks {
    /// 递增syscall次数
    pub fn add_syscall_times(&self,syscall_id:usize){
        let current_task = TASK_MANAGER.get_current_task();
        let mut tasks = self.inner.exclusive_access();
        tasks[current_task].add_syscall_times(syscall_id);
    }
    /// 获取syscall次数
    pub fn get_syscall_times(&self,syscall_id:usize)->u32{
        let current_task = TASK_MANAGER.get_current_task();
        let tasks = self.inner.exclusive_access();
        tasks[current_task].get_syscall_times(syscall_id)
    }
    /// 获取当前app的全部syscall的次数
    pub fn get_total_syscall_times(&self)->[u32;MAX_SYSCALL_NUM] {
        let current_task = TASK_MANAGER.get_current_task();
        let tasks = self.inner.exclusive_access();
        tasks[current_task].get_total_syscall_times()
    }
    /// 开始当前app的统计时间
    pub fn start_current_task_time(&self) {
        let current_task = TASK_MANAGER.get_current_task();
        let mut tasks = self.inner.exclusive_access();
        tasks[current_task].start_task_time();
    }
    /// 获取当前app的运行时间
    pub fn get_current_task_run_time(&self)->usize{
        let current_task = TASK_MANAGER.get_current_task();
        let tasks = self.inner.exclusive_access();
        get_time_ms() - tasks[current_task].get_task_time()
    }
}

impl TaskStatBlock {
    /// 获取当前app的全部syscall的次数
    pub fn get_total_syscall_times(&self)->[u32;MAX_SYSCALL_NUM]{
        self.call_time.clone()
    }
    /// 递增syscall次数
    pub fn add_syscall_times(&mut self,syscall_id:usize){
        self.call_time[syscall_id]+=1;
    }
    /// 获取syscall次数
    pub fn get_syscall_times(&self,syscall_id:usize)->u32{
        self.call_time[syscall_id]
    }
    /// 开始计时
    pub fn start_task_time(&mut self){
        self.start_time = get_time_ms()
    }
    /// 获取开始时间
    pub fn get_task_time(&self)->usize{
        self.start_time
    }
}