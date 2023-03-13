use crate::fs::{make_pipe, open_file, OpenFlags, search_file, add_a_link, rm_a_link};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer, PageTable, VirtAddr, translated_refmut};
use crate::task::{current_task, current_user_token};
use alloc::sync::Arc;

#[repr(C)]
#[derive(Debug)]
pub struct Stat {
	pub dev: u64,
	pub ino: u64,
	pub mode: StatMode,
	pub nlink: u32,
	pad: [u64; 7],
}

bitflags! {
	pub struct StatMode: u32 {
		const NULL = 0;
		const DIR = 0o040000;
		const FILE = 0o100000;
	}
}

/// sys_linkat
pub fn sys_linkat(oldpath: *const u8, newpath: *const u8, _flag: u32) -> isize {
	//let task = current_task().unwrap();
	let token = current_user_token();
	let oldname = translated_str(token, oldpath);
	let newname = translated_str(token, newpath);
	if oldname == newname {
		return -1;
	} else if let Some(_) = search_file(oldname.as_str()) {
		add_a_link(oldname.as_str(), newname.as_str());				
		return 0;
	}
	-1
}

/// sys_unlinkat
pub fn sys_unlinkat(path: *const u8, _flag: u32) -> isize {
	let len = unsafe { (0usize..).find(|i| *((path as usize + *i) as *const u8) == 0).unwrap()};
	let name = unsafe { core::str::from_utf8(core::slice::from_raw_parts(path, len)).unwrap()};
	match search_file(name) {
		Some(_) => rm_a_link(name),
		None => return -1,
	};
	0
}

/// sys_fstat
pub fn sys_fstat(fd: i32, st: *mut Stat) -> isize {
	if fd <= 2 {
		return -1;
	}
	let task = current_task().unwrap();
	let token = current_user_token();
	let st_addr:&mut Stat= PageTable::from_token(token)
				.translate_va(VirtAddr::from(st as *const u8 as usize))
				.unwrap()
				.get_mut();
	if let Some(file) = &task.inner_exclusive_access().fd_table[fd as usize] {
		let file = file.clone();
		file.stat(st_addr);
	} else {
		return -1;
	}
	0
}

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

/// 1. get task control block 
/// 2. apply a pipe
/// 3. return write end and read end
pub fn sys_pipe(pipe: *mut usize) -> isize {
	let task = current_task().unwrap();
	let token = current_user_token();
	let mut inner = task.inner_exclusive_access();
	let (pipe_read, pipe_write) = make_pipe();
	let read_fd = inner.alloc_fd();
	inner.fd_table[read_fd] = Some(pipe_read);
	let write_fd = inner.alloc_fd();
	inner.fd_table[write_fd] = Some(pipe_write);
	*translated_refmut(token, pipe) = read_fd;
	*translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
	0
}

/// 1. get tasj control block
/// 2. check fd validity
/// 3. return duplicated fd 
pub fn sys_dup(fd: usize) -> isize {
	let task = current_task().unwrap();
	let mut inner = task.inner_exclusive_access();
	if fd >= inner.fd_table.len() {
		return -1;
	}
	if inner.fd_table[fd].is_none() {
		return -1;
	}
	let new_fd = inner.alloc_fd();
	inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
	new_fd as isize
}