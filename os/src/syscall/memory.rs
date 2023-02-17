use crate::config::*;
use crate::task::{is_map, map, is_unmap, unmap};

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if start % PAGE_SIZE != 0||
        prot & !0x7 != 0 ||
        prot & 0x7 == 0  ||
        is_map(start, len) == -1 {
            return -1;
        }
    
    map(start, len, prot)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if is_unmap(start, len) == -1 {
        return -1;
    }
    unmap(start, len)
}