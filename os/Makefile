.PHONY: build qemu gdbserver clean fs-img env

MODE = release
QEMU = qemu-system-riscv64
TARGET = riscv64gc-unknown-none-elf
MACHINE = virt
OS_NAME = os.bin
OS_ADDR = 0x80200000 
FS_IMG = ../user/target/$(TARGET)/$(MODE)/fs.img
APPS = ../user/src/bin/*

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed")) || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

fs-img: 
	make -C ../user build
	rm -f $(FS_IMG)
	cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/

user_app:
	make -C ../user build

build: user_app fs-img 
	cargo build --release
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/r_core -O binary target/riscv64gc-unknown-none-elf/release/os.bin

qemu: build
	$(QEMU) -m 256M -M $(MACHINE) \
		-nographic \
		-bios ../bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=$(OS_ADDR) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

		#-kernel target/riscv64gc-unknown-none-elf/release/os.bin
gdbserver:
	$(QEMU) -m 256M -M $(MACHINE) -nographic -kernel target/riscv64gc-unknown-none-elf/release/os.bin -s -S


clean:
	cargo clean
	make -C ../user clean
