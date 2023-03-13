//! Types related to task management
use super::{SignalActions, TaskContext};
use super::{pid_alloc, KernelStack, PidHandle, SignalFlags};
use crate::config::TRAP_CONTEXT;
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{translated_refmut, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use alloc::sync::{Arc, Weak};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;

/// task control block structure
pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
	pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
	pub signals: SignalFlags,
	pub signal_mask: SignalFlags,
	pub handling_sig: isize,
	pub signal_actions: SignalActions,
	pub killed: bool,
	pub frozen: bool,
	pub trap_ctx_backup: Option<TrapContext>,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
	pub fn alloc_fd(&mut self) -> usize {
		if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()){
			fd
		} else {
			self.fd_table.push(None);
			self.fd_table.len() - 1
		}
	}
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner { 
                    trap_cx_ppn, 
                    base_size: user_sp, 
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top), 
                    task_status: TaskStatus::Ready, 
                    memory_set, 
                    parent: None, 
                    children: Vec::new(), 
                    exit_code: 0,
					fd_table: vec![
						// 0 -> stdin
						Some(Arc::new(Stdin)),
						// 1 -> stdout
						Some(Arc::new(Stdout)),
						// 2 -> stderr
						Some(Arc::new(Stdout)),
					],
					signals: SignalFlags::empty(),
					signal_mask: SignalFlags::empty(),
					handling_sig: -1,
					signal_actions: SignalActions::default(),
					killed: false,
					frozen: false,
					trap_ctx_backup: None,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, mut user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** access inner exclusively
		// push the pointers
		user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
		let argv_base = user_sp;
		let mut argv: Vec<_> = (0..=args.len())
			.map(|arg| {
				translated_refmut(
					memory_set.token(),
					(argv_base + arg * core::mem::size_of::<usize>()) as *mut usize,
				)
			})
			.collect();
		*argv[args.len()] = 0;
		// push the argv strings into stack
		for i in 0..args.len() {
			user_sp -= args[i].len() + 1;
			*argv[i] = user_sp;
			let mut p = user_sp;
			for c in args[i].as_bytes() {
				*translated_refmut(memory_set.token(), p as *mut u8) = *c;
				p += 1;
			}
			*translated_refmut(memory_set.token(), p as *mut u8) = 0;
		}
		// aligning to make real machine happy
		user_sp -= user_sp % core::mem::size_of::<usize>();
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize base_size
        inner.base_size = user_sp;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
		// pass the parameters to the task
		trap_cx.x[10] = args.len();
		trap_cx.x[11] = argv_base;
		*inner.get_trap_cx() = *trap_cx;
        // **** release inner automatically
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
		let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
		for fd in parent_inner.fd_table.iter() {
			if let Some(file) = fd {
				new_fd_table.push(Some(file.clone()));
			} else {
				new_fd_table.push(None);
			}
		}
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
					fd_table: new_fd_table,
					signals: SignalFlags::empty(),
					signal_mask: parent_inner.signal_mask,
					handling_sig: -1,
					signal_actions: parent_inner.signal_actions.clone(),
					killed: false,
					frozen: false,
					trap_ctx_backup: None,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}
#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}
