//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod action;
mod signal;
mod context;
mod switch;
mod pid;
mod processor;
mod manager;
#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]
mod task;

use crate::fs::{open_file, OpenFlags};
use alloc::sync::Arc;
pub use context::TaskContext;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager, remove_from_pid2task};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use action::{SignalAction, SignalActions};
pub use signal::{SignalFlags, MAX_SIG};
pub use manager::{add_task, pid2task};
pub use pid::{pid_alloc, KernelStack, PidAllocator, PidHandle};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};

pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();

    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub const IDLE_PID: usize = 0;

use crate::board::QEMUExit;

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            crate::board::QEMU_EXIT_HANDLE.exit_failure();
        } else {
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
    }

	remove_from_pid2task(task.getpid());
    let mut inner = task.inner_exclusive_access();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code = exit_code;

    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }

    inner.children.clear();
    inner.memory_set.recycle_data_pages();
	inner.fd_table.clear();
    drop(inner);
    drop(task);
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
		let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
		let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
	});
}
///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn check_signals_error_of_current() -> Option<(i32, &'static str)> {
	let task = current_task().unwrap();
	let task_inner = task.inner_exclusive_access();
	task_inner.signals.check_error()
}

/// add a signal to the pending list
pub fn current_add_signal(signal: SignalFlags) {
	let task = current_task().unwrap();
	let mut task_inner = task.inner_exclusive_access();
	task_inner.signals |= signal;
}

/// if signal is SIGSTOP, we froze it
/// if signal is SIGCONT, we free the frozen task
/// else we kill the task
pub fn call_kernel_signal_handler(signal: SignalFlags) {
	let task = current_task().unwrap();
	let mut task_inner = task.inner_exclusive_access();
	match signal {
		SignalFlags::SIGSTOP => {
			task_inner.frozen = true;
			task_inner.signals ^= SignalFlags::SIGSTOP;
		}
		SignalFlags::SIGCONT => {
			if task_inner.signals.contains(SignalFlags::SIGCONT) {
				task_inner.signals ^= SignalFlags::SIGCONT;
				task_inner.frozen = false;
			}
		}
		_ => {
			task_inner.killed =true;
		}
	}
}

/// 1. get the signal handler
/// 2. if the sig handler is not none, then clear it, and excute handler
/// 3. store the current context
fn call_user_signal_handler(sig: usize, signal: SignalFlags) {
	let task = current_task().unwrap();
	let mut task_inner = task.inner_exclusive_access();

	let handler = task_inner.signal_actions.table[sig].handler;
	if handler != 0 {
		task_inner.handling_sig = sig as isize;
		task_inner.signals ^= signal;

		let mut trap_ctx = task_inner.get_trap_cx();
		task_inner.trap_ctx_backup = Some(*trap_ctx);
		trap_ctx.sepc = handler;
		trap_ctx.x[10] = sig;
	} else {
		println!("[K] task/call_user_signal_handler: default action: ignore it or kill process");
	}
}

/// if the signal exists and is not masked
/// 	
/// 
fn check_pending_signals() {
	for sig in 0..(MAX_SIG + 1) {
		let task = current_task().unwrap();
		let task_inner = task.inner_exclusive_access();
		let signal = SignalFlags::from_bits(1 << sig).unwrap();
		if task_inner.signals.contains(signal) && (!task_inner.signal_mask.contains(signal)) {
			let mut masked = true;
			let handling_sig = task_inner.handling_sig;
			if handling_sig == -1 {
				masked = false;
			} else {
				let handling_sig = handling_sig as usize;
				if !task_inner.signal_actions.table[handling_sig]
					.mask
					.contains(signal)
				{
					masked = false;
				}
			}
			if !masked {
				drop(task_inner);
				drop(task);
				if signal == SignalFlags::SIGKILL
					|| signal == SignalFlags::SIGSTOP
					|| signal == SignalFlags::SIGCONT
					|| signal == SignalFlags::SIGDEF 
				{
					call_kernel_signal_handler(signal);
				} else {
					call_user_signal_handler(sig, signal);
					return;
				}
			}
		}
	}
}

pub fn handle_signals() {
	loop {
		check_pending_signals();
		let (frozen, killed) = {
			let task = current_task().unwrap();
			let task_inner = task.inner_exclusive_access();
			(task_inner.frozen, task_inner.killed)
		};
		if !frozen || killed {
			break;
		}
		suspend_current_and_run_next();
	}
}