use super::TaskContext;

const MAX_SYSCALL_NUM: usize = 512;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_info: TaskInfo,
    pub last_time: usize,
}

#[derive(Copy, Clone)]
pub struct TaskInfo {
    pub id: usize,
    pub status: TaskStatus,
    pub call: [SyscallInfo; MAX_SYSCALL_NUM],
    pub time: usize,
}

#[derive(Copy, Clone)]
pub struct SyscallInfo {
    pub id: usize,
    pub times: usize,
}


#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

impl TaskInfo {
    pub fn new() -> Self {
        Self {
            id: usize::MAX,
            status: TaskStatus::UnInit,
            call: [ SyscallInfo::new(); MAX_SYSCALL_NUM],
            time: 0,
        }
    }
}

impl SyscallInfo {
    pub fn new() -> Self {
        Self {
            id: usize::MAX,
            times: 0
        }
    }
}