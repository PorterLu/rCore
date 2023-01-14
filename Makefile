.PHONY: build qemu gdbserver clean

QEMU = qemu-system-riscv64
TARGET = riscv64gc-unknown-none-elf
BIOS = bootloader/rustsbi-qemu.bin
MACHINE = virt
OS_NAME = os.bin
OS_ADDR = 0x80200000 

build:
	cargo build --release
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/blog_os -O binary target/riscv64gc-unknown-none-elf/release/os.bin

qemu:
	$(QEMU) -m 256M -M $(MACHINE) -nographic -kernel target/riscv64gc-unknown-none-elf/release/os.bin

gdbserver:
	$(QEMU) -m 256M -M $(MACHINE) -nographic -kernel target/riscv64gc-unknown-none-elf/release/os.bin -s -S


clean:
	cargo clean
