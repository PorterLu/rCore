
use crate::bitflags;

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