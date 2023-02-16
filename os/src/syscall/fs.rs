use crate::batch::{check_bound};
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len)};
            let str = core::str::from_utf8(slice).unwrap();
            if check_bound(str.as_ptr() as usize) {
                print!("{}", str);
                len as isize
            } else {
                panic!("Only support for own space");
            }
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
