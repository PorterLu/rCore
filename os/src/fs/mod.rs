//! File system in os
mod inode;
mod stdio;
use crate::syscall::fs::Stat;

use crate::mm::UserBuffer;
/// File trait
pub trait File: Send + Sync {
    /// If readable
    fn readable(&self) -> bool;
    /// If writable
    fn writable(&self) -> bool;
    /// Read file to `UserBuffer`
    fn read(&self, buf: UserBuffer) -> usize;
    /// Write `UserBuffer` to file
    fn write(&self, buf: UserBuffer) -> usize;
	/// state
	fn stat(&self, st: &mut Stat) -> usize;
}

pub use inode::{list_apps, open_file, OSInode, OpenFlags, search_file, add_a_link, rm_a_link};
pub use stdio::{Stdin, Stdout};
