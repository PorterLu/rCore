# rCore第二章

## 概述

​	这一节，我们将放置程序到`User`模式下执行，同时通过批处理程序逐个加载应用程序。程序设计的要点是：

* 应用程序的内存布局
* 应用程序发出的系统调用

## 项 目结构

​	批处理系统会加载`user/src/bin` 下的多个程序进行运行，同时每个程序在源代码中会引入外部库:

```rust
#[macro_use]
extern crate user_lib;
```

​	这个外部库其实就是`user`目录下的`lib.rs`以及它引用的若干子模块，子所以叫`user_lib`是因为`lib.rs`所在的目录是`user`，并且我们在`user/Cargo.toml`中对于库的名字进行了设置，`name = "user_lib"`。它作为`bin`目录下的源程序所依赖的用户库，等价编程语言提供的标准库。

```rust
#[no_mangle]
#[link_section = '.text.entry']
pub extern "C" fn _start() -> !{
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}
```

​	我们使用宏将`_start`放到`.text.entry`段中，方便后续链接的时候调整它的位置，使得它能够作为用户库的入口。进入用户库的入口之后，首先手动初始化`.bss`段，之后调用`main`函数，最后进行`exit`。在`lib.rs`中还存在一个`main`函数：

```rust
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32{
    panic!("Cannot find main!");
}
```

​	第一行，我们使用宏将函数符号`main`定义为弱链接，这样最后链接的时候，虽然在`lib.rs`和`bin`目录下的某个应用都有`main`符号，但是由于`lib.rs`中的`main`是弱链接，这样链接器会使用`bin`目录的`main`。为了支持这样的链接操作，我们需要在`lib.rs`的开头加入：

```rust
#![feature(linkage)]
```

## 内存布局

在`user/.cargo/config`中，我们和第一章一样设置链接的链接脚本`user/src/linker.ld`，在其中我们做的了如下的工作：

* 将程序的起始物理地址调整为`0x80400000`，三个应用程序都要被加载这个物理地址上运行。
* `_start`所在`.text.entry`放在整个程序的开头，也就是说批处理程序在加载完程序后就会跳转到`0x80400000`就已经进入到用户库的入口点，并会在初始化之后跳转到应用程序主逻辑。
* 提供了最终可执行文件的`.bss`起始地址和终止地址。

## 系统调用

在子模块`syscall`中，应用程序通过`ecall`调用批处理程序提供的接口，由于应用程序提运行在用户态，所以在`ecall`会触发`Environment call from U-mode`的异常，并且陷入到`S`模式的处理程序。

```rust
///功能：将内存中缓冲区的数据写入文件
///参数：`fd`  表示待写入文件的文件描述符
///		`buf` 表示内存中缓冲区的起始地址
///		`len` 表示内存中缓冲区的长度
///返回值：返回成功写入的长度
///syscall ID: 64
fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize;

///功能：退出应用程序并将返回值告知批处理系统
///参数：`xstate` 表示应用程序的返回值
///syscall ID:93
fn sys_exit(xstate: usize) -> !;
```

​	系统调用实际上是汇编指令级的二进制接口，我们需要按照`RISC-V`在和合适的寄存器放弃系统的调用的参数，

```rust
// user/src/syscall.rs
use core::arch::asm;
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe{
        asm!(
        	"ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1];
            in("x12") args[2];
            in("x17") id
        );
    }
    ret
}
```

​	第3行，我们将所有的系统调用都封装成`syscall`函数，可以看到它支持传入`syscall ID` 和 3个参数。作为程序员我们并不知道只有编译器才知道的信息，我们只能在编译器的帮助下完成变量到寄存器的绑定。现在来看`asm!`宏的格式，可以看到我们使用`inlateout`,在这里表示它既作为输入又作为输出，使用{in_var} => {out_var}的格式。

​	我们对`sys_write` 和 `sys_exit`进行封装：

```rust
// user/src/syscall.rs
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(xstate: i32) -> isize{
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
}
```

​	这里使用一个`&[u8]`切片类型来描述缓冲区，这是一个胖指针，里面既包含了缓冲区的起始地址，还包含了缓冲区的长度。我们可以 通过`as_ptr` 和 `len`方法去除它们作为独立的系统调用参数。

​	我们对上述的系统调用在`user_lib`中进行进一步地封装，从而更加接近`Linux`等平台的实际系统调用接口：

```rust
// user/src/lib.rs
use syscall::*;

pub fn write(fd:usize, buf:&[u8]) -> isize{ sys_write(fd, buf)}
pub fn exit(exit_code: i32) -> isize{ sys_exit(exit_code)}
```

​	我们把`console`子模块中`Stdout::write_str` 改成基于`write`的实现，且传入的`fd`参数设置为1，我们把`console`子模块中的`Stdout::write_str` 改成基于`write`实现，且传入的`fd`设置为1, 它代表标准输出，也就是输出到屏幕。目前不考虑其他`fd`的情况，应用程序的`println!` 宏借助系统调用也就变得可用了。

```rust
// user/src/console.rs
const STDOUT: usize = 1;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result{
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}
```

​	`exit`接口则在用户库中的`_start`内使用，当应用程序主逻辑`main`返回后，使用它退出引用程序并将返回值告知底层的批处理系统。

​	而应用程序自动构建的过程，只需要在`user`目录下执行`make build`即可：

1. 对于`src/bin`下的每个应用程序，在`target/riscv64gc-unknown-none-elf/release`目录下生成一个同名的`ELF`可执行程序。
2. 使用`objcopy`将二进制工具从上一步中生的`ELF`文件删除`ELF header`和符号得到二进制镜像文件，他们将在合适的实际被加载经内存。

## 实现操作系统前执行应用程序

​	`Qemu`还支持运行`RISC-V 64`用户程序的半系统模拟器`qemu-riscv64`，如果想让用户态的程序在`qemu-riscv64`模拟器上和我们自己写的`OS`上执行效果一直，要做到二者系统调用接口是一致的。假定我们已经完成编译生成了`ELF`可执行程序：

```rust
// user/src/bin/03priv_inst.rs
use core::arch::asm;
#[no_mangle]
fn main() -> i32{
    println!("Try to execute privileged instruction in U Mode");
    println!("Kernel should kill this application!");
    unsafe{
        asm!("sret");
    }
    0
}

// use/src/bin/04priv_csr.rs
use riscv::register::sstatus::{self, SPP};
#[no_mangle]
fn main() -> i32{
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe{
        sstatus::set_spp(SPP::User);
    }
    0
}

```

## 将应用程序链接到内核

### 概述

​	在批处理程序中，我们需要实现应用的加载功能，下面是两种方式：

* 静态绑定：通过一定的编程技巧，把多个应用程序代码和批处理操作系统代码绑定在一起。
* 动态架子啊：基于静态编码留下的绑定信息，操作系统可以找到每个应用程序文件二进制代码的起始位置和长度，并加载到内存中运行。

### 布局

​	在本章节中，我们把应用程序的二进制镜像文件（从`ELF`格式可执行文件剥离元数据），作为内核的数据段链接到内核中，因此内核需要知道其中的应用程序数量和它们的位置，这样才能够在运行时对它们进行管理并能够加载都物理内存。

在`main.rs`中，有如下的语句：

```assembly
global_asm!(include_str!("link_app.S"));
```

​	`link_app.S`是在编译时自动生成的，

```assembly
# os/src/link_app.S

	.align 3
	.section .data
	.global _num_app
_num_app:
	.quad 5
	.quad app_0_start
	.quad app_1_start
	.quad app_2_start
	.quad app_3_start
	.quad app_4_start
	.quad app_4_end
	
	.section .data
	.global app_0_start
	.global app_0_end
app_0_start:
	.incbin "../user/target/riscv64gc-unknown-none-elf/release/00hello_world.bin"
app_0_end:
	
    .section .data
    .global app_1_start
    .global app_1_end
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01store_fault.bin"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
app_2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/02power.bin"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
app_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/03priv_inst.bin"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
app_4_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/04priv_csr.bin"
app_4_end:
```

​	可以看到第15行开始的五个数据段分别插入了五个应用程序的二进制镜像，并且各自有一对全局符号`app_*_start` , `app_*_end`， 数据段中相当于有了一个64位整数数组，数组中的一个元素表示应用程序的数量，后面则按照顺序放置每个应用程序的起始地址，最后一个元素放置最后一个程序的结束位置。

### 找到并加载应用程序二进制代码

​	我们在`os`的`batch`子模块中实现一个应用管理器，它的主要功能是：

* 保存应用数量和各自的位置信息，以及当前执行到第几个应用
* 根据应用程序位置信息，初始化好应用所需内存空间，并加载应用执行。

​	应用管理器`AppManager`结构体的定义如下：

``` rust
// os/src/batch.rs

struct AppManager{
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}
```

### Rust所有权和借用检查

​	用一句话进行概括就是 **值** 在同一时间只能绑定到一个 **变量**。这里的“值”指的是存储在内存中固定位置，且属于某种特定类型的数据；而变量就是我们`Rust`代码中通过`let`声明的局部变量或者函数的参数等，变量的类型和值的类型相匹配。这种情况下，我们值的所有权属于它被绑定到的变量，且变量可以作为访问/控制绑定到它上面的值的一个媒介。变量可以将所有权转移给其他变量，或者当变量退出其作用域之后，它拥有的值也将被销毁，这也意味值占用的内存或者资源将被回收。

​	有的时候，特别是函数调用的情况下，我们不希望当前上下文中的值的所有权被转移到其他上下文中，因此可以使用引用的概念，`Rust`使用`&`或者`mut&`表示引用和可变引用。

* 不可变/可变引用的生存周期不能超出它们借用的值的声明周期。
* 同一时间，借用同一值的可变和不可变不能共存。
* 同一时间，借用同一个值的不可变引用可能存在多个，但是可变引用只能有一个。

​	第一条是很好理解的，只有值合法，引用才有意义，如果发生指针悬垂，即我们尝试在一个函数中返回函数中声明的局部变量的引用，并且调用试图通过引用访问已经被销毁的局部变量，这样将会产生未定义的错误。第二和第三条主要防止对同一个值有多个引用进行读写。

​	对于借用方式运行时可变的情况，我们可以将借用检出推迟到运行时，这种称为运行时借用检查，这种情况下值的借用状态会占用额外的一部分存储空间，运行时还会有额外的代码进行借用合法性检查，这是为灵活性付出的开销。当无法通过借用检查时，将会产生一个不可恢复的错误。具体来说，我们使用`RefCell`包括可被借用的值，随后调用`borrow`和`borrow_mut`即可以发出一个对值的不可变/可变借用，终止借用，我们可以手动销毁也可以等待它们被自动销毁。

​	

