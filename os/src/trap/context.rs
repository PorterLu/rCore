use riscv::register::sstatus::{self, Sstatus, SPP};

//Trap Context
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    //set new stack_pointer to x2(sp register)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    //set 0 to general register, set sepc to app_entry, sstatus'spp to User, 
    //return the context
    pub fn app_init_context(entry: usize, sp:usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}