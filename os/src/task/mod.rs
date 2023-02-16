mod context;
mod switch;

#[allow(clippy::module_inception)]
pub mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus, TaskInfo};
use crate::timer::get_time;

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock{
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            task_info: TaskInfo::new(),
            last_time: 0,
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
            task.task_info.status = TaskStatus::Ready;
            task.task_info.id = i;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> !{
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.task_info.status = TaskStatus::Running;
        task0.last_time = get_time();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
        inner.tasks[current].task_info.status = TaskStatus::Ready;
        inner.tasks[current].task_info.time += get_time() - inner.tasks[current].last_time;
    }

    fn mark_current_exited(&self){
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
        inner.tasks[current].task_info.status = TaskStatus::Exited;
        inner.tasks[current].task_info.time += get_time() - inner.tasks[current].last_time;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current+1..current+self.num_app+1)
            .map(|id| id%self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self){
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.tasks[next].task_info.status = TaskStatus::Running;
            inner.tasks[next].last_time = get_time();
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            println!("All applications completed!");
            use crate::board::QEMUExit;
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
    }

    fn set_syscall_record(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_info.call[syscall_id].id = syscall_id;
        inner.tasks[current].task_info.call[syscall_id].times += 1;
        drop(inner);
    }

    fn get_task_info(&self, id: usize, task_info: *mut TaskInfo) -> isize {
        if id < MAX_APP_NUM {
            let inner = self.inner.exclusive_access();
            let current = inner.current_task;
            unsafe {
                (*task_info).id = id;
                (*task_info).status = inner.tasks[current].task_status;
                (*task_info).time = inner.tasks[current].task_info.time;
                (*task_info).call = inner.tasks[current].task_info.call;
            }
            drop(inner);
            0
        } else {
            -1
        }
    }

}

//TASK_MANAGER is visible to the same level function
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn set_syscall_record(syscall_id: usize) {
    TASK_MANAGER.set_syscall_record(syscall_id);
}

pub fn get_task_info(id: usize, task_info: *mut TaskInfo) -> isize {
    TASK_MANAGER.get_task_info(id, task_info)
}