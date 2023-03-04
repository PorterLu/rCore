use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

/// sys_write:
/// the task own a file table:
///  	if file id is bigger than fd_table_len, it means a error.
/// 	else if the file is writable
///  		call file write function
/// 	otherwise
/// 		a error
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
	let task = current_task().unwrap();
	let inner = task.inner_exclusive_access();
	if fd >= inner.fd_table.len() {
		return -1;
	}
	if let Some(file) = &inner.fd_table[fd] {
		if !file.writable() {
			return -1;
		}
		let file = file.clone();
		drop(inner);
		file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
	} else {
		-1
	}
}

/// sys_read:
/// the task own a file table:
/// 	if file id is bigger than fd_table_len, it means a error
/// 	else if the file is readable
/// 		call the file's read function 
/// 	otherwise
/// 		a error
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
	let token = current_user_token();
	let task = current_task().unwrap();
	let inner = task.inner_exclusive_access();
	if fd >= inner.fd_table.len() {
		return -1;
	}
	if let Some(file) = &inner.fd_table[fd] {
		let file = file.clone();
		if !file.readable() {
			return -1;
		}
		drop(inner);
		file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
	} else {
		-1
	}
}

/// sys_open
/// 	get file according name, put inode into fd_table
pub fn sys_open(path: *const u8, flags: u32) -> isize {
	let task = current_task().unwrap();
	let token = current_user_token();
	let path = translated_str(token, path);
	if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
		let mut inner = task.inner_exclusive_access();
		let fd = inner.alloc_fd();
		inner.fd_table[fd] = Some(inode);
		fd as isize
	} else {
		-1
	}
}

/// sys_close
/// 	remove inode from fd_table 
pub fn sys_close(fd: usize) -> isize {
	let task = current_task().unwrap();
	let mut inner = task.inner_exclusive_access();
	if fd >= inner.fd_table.len() {
		return -1;
	}
	if inner.fd_table[fd].is_none() {
		return -1;
	}
	inner.fd_table[fd].take();
	0
}

