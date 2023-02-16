pub const MAX_SYSCALL_NUM: usize = 512;

#[derive(Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

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
