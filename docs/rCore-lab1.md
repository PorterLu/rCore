# rCore 第一章

[TOC]

## 多层抽象是必须的吗

​	除了最上层的应用程序和最下层的硬件平台必须存在，作为中间层的函数库和操作系统内核并不是必须存在的：它们都是对下层的资源进行了抽象，为上层提供了一个执行环境。抽象的优点在于它上层以较小的代价获取需要的功能，并提供保护。抽象同时也是一种限制，会丧失一些灵活性。过多的抽象和过少的抽象，都是不合适的。所以理解应用的需求就变得很重要。

* 如果没有库函数和操作系统内核，那么我们就需要手写汇编代码来控制硬件，这种方式具有最高的灵活性，抽象能力最低，基本等于编写汇编代码来控制硬件。
* 如果仅仅存在库函数，我们就不需要操作系统的通用抽象。在单一嵌入式场景就可能会出现这种情况。
* 如果存在库函数和操作系统内核，这意味着应用需求比较多样，会需要并发执行。常见的通用操作系统如Windows/Linux都支持并发运行不同的程序。

​	我们通过一个三元组来表示一个目标平台（CPU架构，CPU厂商，操作系统和运行时库）。我们运行如下的命令：`rustc --version --verbose`

```
host: x86_64-unknown-linux-gnu
```

​	说明这是一个`x64_64`的CPU，厂商未知，操作系统是`linux`，运行时库`GNU libc`。

​	下面我们选择使用后`riscv64gc-unknown-none-elf`进行实现，这里的`G`是`IMAFD`的总称。接下来的开发将不再使用操作系统，Rust有一个经过裁剪后的`std`库，叫做`core`，这个库不需要任何操作系统的支持。

## 移除标准库

​	本章的目标是构造一个内核最小执行环境，为此我们首先要移除标准库的以来，我们需要添加能够支持裸机应用的库操作系统`LibOS`。`LibOS`以库函数的形式存在，为应用程序提供操作系统的基本功能。它最早来自于外核的研究，把传统的单体内核分为两个部分，一部分以库操作系统的形式存在，和应用程序紧耦合实现传统的操作系统的抽象；另外一部分仅仅专注于最基本的安全复用物理硬件的机制上，来给LibOS提供基本的硬件访问服务。这样就可以针对应用程序特征定制LibOS，来达到高性能的目的。

​	使用`#![no_std]`可以使得去除标准库的依赖，但是我们还需要实现`panic!`宏，`#[panic_handler]`是一种编译指导属性，用于标记核心库`core`中的`panic!`宏需要对接的函数（该函数实现对致命错误的具体处理）。该编译指导属性所标记的函数需要具有`fn(&PanicInfo) -> !`函数签名，函数可以通过`PanicInfo`数据结构获取错误信息的相关信息。这样Rust编译器就可以把核心库中的`core`中的`panic!`宏和`#[panic_handler]`指向的`panic`函数实现合并在一起。

​	进行编译，编译器提醒我们，我们缺少一个`start`语义项，在进入`main`函数之前，需要进行一个`start`进行初始化，由于我们禁用了标准库，编译器就找到这些功能了。最简单的方案就是不让编译器使用这些功能，我们在`main.rs`的开头加入`#![no_main]`通知编译器我们没有一般意义的`main`函数。

​	我们可以使用绝对路径或相对路径来引用其他模块或当前模块的内容。参考上面的` use core::panic::PanicInfo; `，类似 C++ ，我们将模块的名字按照层级由浅到深排列，并在相邻层级之间使用分隔符 :: 进行分隔。路径的最后一级（如 `PanicInfo`）则表示我们具体要引用或访问的内容，可能是变量、类型或者方法名。当通过绝对路径进行引用时，路径最开头可能是项目依赖的一个外部库的名字，或者是` crate` 表示项目自身的根模块。在后面的章节中，我们会多次用到它们

​	我们可以通过各种工具来分析目前的程序：`file`, `rust-readobj`, `rust-objdump`。通过`file`工具分析它好像是一个合法的程序，但是通过`rust-readobj`发现这是一个`Entry`为`0`的程序。

## 内核的第一条指令

### 了解QEMU模拟器

​	接下来我们即将将我们的内核对接到`Qemu`模拟器上，使得模拟器可以正确运行内核的第一条指令。本实验中，我们使用`qemu-system-riscv64`来模拟一台64位的`RISC-V`架构的计算机。我们如下的命令启动我们的内核：

```
qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
```

​	这里的`-bios`选项可以设置`Qemu`模拟器开机时用初始化的引导加载程序，我们这里使用编译的`rustsbi-qemu.bin`。`-device`中的`loader`选项可以在`Qemu`模拟器开机前将一个宿主机器上的文件加载到`Qemu`的物理内存中的指定位置，`file`可以用指定该文件，`addr`指定加载后的地址。

### QEMU启动流程

​	`virt`, 物理内存的其实物理地址为`0x80000000`,物理内存的默认大小为`128MB`，这个可以通过`-m`进行配置。如果使用上述的命令启动`Qemu`那么在`Qemu`执行任何指令之前，首先将`rustsbi-qemu.bin`加载到`0x80000000`，同时将`os.bin`加载到物理地址`0x80200000`。下面将介绍多阶段引导，

* 第一阶段，将必要的文件加载到`Qemu`的物理内存后，`Qemu`的`PC`值设置为`0x1000`，在执行几条指令后，将跳转到`0x80000000`，进入第二阶段。
* 第二阶段，由于`Qemu`的第一阶段将固定跳转到`0x80000000`，这就是我们的`bootloader`即`rustsbi-qemu.bin`第一条指令的地址。`bootloader`将进行一系列的初始化，之后将跳转到内核镜像的位置。对于不同的`bootloader`而言，可能下一阶段的软件的位置可能不固定，它可以是约定好的固定的值，也可以是运行时动态获取的值。我们这里是约定好的固定的`0x80200000`。
* 第三阶段，为了正确和上一阶段进行对接，我们必须确保内核的第一条指令位于物理地址的`0x80200000`处。

### 程序内存布局于与编译流程

#### 程序内存布局

​	![](C:\Users\12582\Desktop\memory_layout.png)

我们对数据部分的段，进行细致的分析：

* 已经初始化数据段保存程序中内些已经初始化的全局数据，分为`.rodata`和`.data`两部分，前者只存放只读全局数据，比如一些常数或者是常量字符串等；而后者存放可修改的全局数据。
* 未初始化数据段`.bss`保存程序中那些未初始化的全局数据结构，通常加载代为初始化为0。
* 堆，这个区域用于程序运行时动态分配。
* 栈，用于函数上下文的保存和回复，还有局部变量的存储。

#### 编译的过程

​	从源代码到可执行的二进制程序可以被细分为多个阶段。编译的过程是比较熟悉的，汇编器输出后各个`object`文件是独立的，链接器要这些文件链接成一个整体的布局。在此期间链接器主要完成两件事情：

* 第一件事情就是将来自不同目标文件的段在目标内存中进行重新布局。如下所示，在链接的过程中，分别来自来自两个`object`文件的节，按照功能进行分类，功能类似的节被分到同一个段中。

![](C:\Users\12582\Desktop\linked.png)

* 第二个事情是将符号替换为地址，在机器码级别是通过地址进行访问的，`object`文件给出模块的内部内存布局，此时模块外部符号的地址无法确定。我们需要将这些外部符号记录下来，放在一个符号表中。当两个`object`文件被链接到一起，它们的内存布局会被合并，也就意味这个各个段的位置就已经被确定下来，模块1使用的模块2的符号的地址就可以被确定下来，这个过程被称为重定位。

#### 编写内核的第一条指令

```assembly
# os/sr/entry.asm
	.section .text.entry
	.global _start
_start:
	li x1, 100
```

​	接着在`main.rs`中添加这些汇编代码，代码如下：

```rust
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
```

#### 调整内核的内存布局

​	因为链接器默认的内存呢布局不能符合我们的要求，实现于`Qemu`的正确对接，我们可以通过链接脚本来调整来调整链接器的行为，使得最终的可执行文件的内存布局符合我们的预期。

```
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
	. = BASE_ADDRESS;
	skernel = .;
	
	stext = .;
	.text : {
		*(.text.entry)
		*(.text .text.*)
	}
	
	. = ALIGN(4K);
	etext = .;
	srodaa = .;
	.rodata : {
		*(.rodata .rodata.*)
		*(.srodata .srodata.*)
	}
	
	. = ALIGN(4K);
	erodata = .;
	sdata = .;
	.data : {
		*(.data .data.*)
		*(.sdata .sdata.*)
	}
	
	. = ALIGN(4K);
	edata = .
	.bss : {
		*(.bss.stack)
		sbss = .;
		*(.bss .bss.*)
		*(.sbss .sbss.*)
	}
	
	. = ALIGN(4K);
	ebss = .;
	ekernel = .;
	
	/DISCARD/:{
		*(.eh_frame)
	}
}
```

​	第一行我们设置了目标平台，之后是我们的入口点。之后是一个变量，用于设置为我们的基地址。之后是段布局的说明，将所有目标文件中的段汇聚到目标文件的段中。但是我们不能将上面编译得到的结果给`Qemu`, 因为它还有除了数据段和代码段之外的一些元数据，我们需要将其去除。

```shell
rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os -O binary target/riscv64gc-unknown-none-elf/release/os.bin
```

#### 基于GDB的启动验证

```
qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-elf/release/os.bin,addr=0x80200000\
	-s -S
```

​	`-s`可以使`Qemu`监听本地TCP端口1234等待GDB客户端连接，`-S`使得在收到`GDB`请求后再开始运行。

​	之后再启动一个`GDB`连接到`Qemu`, 我们使用如下的命令：

```shell
$ riscv64-unknown-elf-gdb \
	-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
	-ex 'set arch riscv:rv64'
	-ex 'target remote localhost:1234'
```

​	启动就可以看到`PC`已经被初始化为`0x1000`，我们使用`x/10i $pc`可以看见固件的代码，这里固件执行几条语句后直接跳转到`0x80000000`, 我们打下一个断点`b *0x80200000`这样就可以直接跳转内核的地址，可以看见我们加入的指令`li ra, 100`。

### 调用规范

![](C:\Users\12582\Desktop\call_convention.png)

* `ra` 寄存器是调用者保存的，在每次调用前进行保存，调用结束后进行恢复。
* `sp` 是被调用者保存寄存器
* `fp` 是被调用者保存寄存器，也可以作为栈帧寄存器
* `x3` 和 `x4` 作为 `gp` 和 `tp` 寄存器，在程序运行期间是不会变化的。

![](C:\Users\12582\Desktop\stack_frame.png)

​	我们的函数调用是基于栈来实现的，通过栈帧寄存器可以回溯整个调用链。

### 分配并使用启用栈

​	我们在`entry.asm`中进行栈的初始化，代码如下：

```
# os/src/entry.asm
	.section .text.entry
	.global _start
_start:
	la sp, boot_stack_top
	call rust_main
	
	.section .bss.stack
	.global boot_stack
boot_stack:
	.space 4096 * 16
	.global boot_stack_top
boot_stack_top
```

​	之后转移到`Rust`代码的入口处：

```rust
// os/src/main.rs
#[no_mangle]
pub fn rust_main() -> !{
    loop{}
}
```

​	这里使用`#[no_mangle]` 标记`rust_main` , 防止连接时找不到`main`而出现错误。我们需要完成对`.bss` 段的清零，任何被分配到`.bss`段的变量都要被清0。

```rust
// os/src/main.rs
#[no_mangle]
pub fn rust_main() -> !{
    clear_bss();
    loop{}
}

fn clear_bss(){
    extern "C"{
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a|{
        unsafe {(a as *mut u8).write_volatile(0)}
     });
}
```

​	下面解释`clear_bss`中代码的意思，首先在`extern "C"`引用了外部`C`函数的接口，这里将位置标志转化为`usize`获取其对应的地址，这样就可以知道`.bss`段两端的地址。

​	下面的代码将地址转化为以裸指针，之后使用`for each`进行映射，每一个地址在`unsafe`块内都被初始化为0。

### 基于SBI完成输出

#### 使用RustSBI的服务

​	之前我们使用`RustSBI` 启动，它会在计算机启动时进行初始化，并将控制权交给内核。但是实际上，它还有另外一个作用，就是当内核发出请求时，RustSBI会响应内核的请求。

```rust
// os/src/main.rs
mod sbi;

use core::arch::asm;
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1:usize, arg2:usize) -> usize {
    let mut ret;
    unsafe{
        asm!(
        	"ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret 
}
```

​	我们将内核与`RustSBI`通信的相关功能实现在子模块`sbi`中，我们需要在`main.rs`中加入`mod sbi`将该子模块加入我们的项目。`inlateout`用于表示先输入后输出，`which`表示请求`RustSBI`的服务的类型，`arg0` ~ `arg2` 表示传递给`RustSBI` 的3个参数，而`RustSBI`在将请求处理完毕后，会给内核返回值。

```rust
// os/src/sbi.rs
#![allow(unused)]
const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;
```

​	我们对输出一个字符的功能进行封装，函数如下：

```rust
// os/src/sbi.rs
pub fn console_putchar(c: usize){
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}
```

​	这里我们没有使用`sbi_call`的返回值，接着如果要使用关机服务，可以封装关机服务：
```rust
pub fn shutdown() -> !{
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
```

#### 实现格式化输出

`console_putchar`的功能过于受限，如果想打印一行的`Hello world!` 的话需要多次进行调用，如果想使用`println!` 宏，我们需要编写自己的`println!`宏。

```rust
use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout{
    fn write_str(&mut self, s:&str) -> fmt::Result{
        for c in s.chars(){
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments){
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
```

​	我们需要在`console`子模块中编写`println!`宏，结构体`Stdout`不包含任何字段，因此它被称为类单元结构。`core::fmt::Write` trait包含一个用来实现`println!`宏很好用的`write_fmt` 方法，为此我们`Stdout` 实现 `Write` trait。