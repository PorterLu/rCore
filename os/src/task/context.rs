#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    /// return address 
    pub ra: usize,
    ///kernel stack pointer of app
    pub sp: usize,
    /// callee save register: s(0-11)
    s: [usize; 12],
}

impl TaskContext {
    ///init task context
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// set task context
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}