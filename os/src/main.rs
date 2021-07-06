//! # 全局属性
//!   禁用标准库
#![no_std]
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口
#![no_main]
//! # 一些 unstable 的功能需要在 crate 层级声明后才可以使用
#![feature(llvm_asm)]
//!   内嵌整个汇编文件
#![feature(global_asm)]
//!   panic! 时，获取其中的信息并打印
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
//!
//!   允许使用 naked 函数，即编译器不在函数前后添加出入栈操作。
//!   这允许我们在函数中间内联汇编使用 `ret` 提前结束，而不会导致栈出现异常
#![feature(naked_functions)]
//!
//!   允许将 slice 填充值
#![feature(slice_fill)]

use crate::sbi::shutdown;

#[macro_use]
mod console;
mod interrupt;
mod memory;
mod panic;
mod process;
mod sbi;

extern crate alloc;

use alloc::sync::Arc;
use process::*;

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

/// Rust 的入口函数
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Boot success!\n");
    println!("Hello rCore-Tutorial!");

    // 初始化各种模块
    interrupt::init();
    memory::init();

    // interrupt_test();
    dynamic_memory_alloc_test();
    physical_memory_alloc_test();

    // kernel_remap_test();  // conflict with thread_test()
    thread_test();

    shutdown();
    // loop{}
}

fn thread_test() {
    {
        let mut processor = PROCESSOR.lock();
        // 创建一个内核进程
        let kernel_process = Process::new_kernel().unwrap();

        // 为这个进程创建多个线程，并设置入口均为 sample_process，而参数不同
        for i in 1..9usize {
            processor.add_thread(create_kernel_thread(
                kernel_process.clone(),
                sample_process as usize,
                Some(&[i]),
            ));
        }
    }

    extern "C" {
        fn __restore(context: usize);
    }
    // 获取第一个线程的 Context
    let context = PROCESSOR.lock().prepare_next_thread();
    // 启动第一个线程
    unsafe { __restore(context as usize) };
}
fn sample_process(id: usize) {
    println!("hello from kernel thread {}", id);
}

/// 创建一个内核进程
pub fn create_kernel_thread(
    process: Arc<Process>,
    entry_point: usize,
    arguments: Option<&[usize]>,
) -> Arc<Thread> {
    // 创建线程
    let thread = Thread::new(process, entry_point, arguments).unwrap();
    // 设置线程的返回地址为 kernel_thread_exit
    thread
        .as_ref()
        .inner()
        .context
        .as_mut()
        .unwrap()
        .set_ra(kernel_thread_exit as usize);

    thread
}

// /// 创建一个用户进程，从指定的文件名读取 ELF
// pub fn create_user_process(name: &str) -> Arc<Thread> {
//     // 从文件系统中找到程序
//     let app = ROOT_INODE.find(name).unwrap();
//     // 读取数据
//     let data = app.readall().unwrap();
//     // 解析 ELF 文件
//     let elf = ElfFile::new(data.as_slice()).unwrap();
//     // 利用 ELF 文件创建线程，映射空间并加载数据
//     let process = Process::from_elf(&elf, true).unwrap();
//     // 再从 ELF 中读出程序入口地址
//     Thread::new(process, elf.header.pt2.entry_point() as usize, None).unwrap()
// }

/// 内核线程需要调用这个函数来退出
fn kernel_thread_exit() {
    // 当前线程标记为结束
    PROCESSOR.lock().current_thread().as_ref().inner().dead = true;
    // 制造一个中断来交给操作系统处理
    unsafe { llvm_asm!("ebreak" :::: "volatile") };
}

fn interrupt_test() {
    // 中断测试
    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    };
}

fn dynamic_memory_alloc_test() {
    // 动态内存分配测试
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let v = Box::new(5);
    assert_eq!(*v, 5);
    core::mem::drop(v);

    let mut vec = Vec::new();
    for i in 0..10000 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 10000);
    for (i, value) in vec.into_iter().enumerate() {
        assert_eq!(value, i);
    }
    println!("==> heap test passed");
}

fn physical_memory_alloc_test() {
    // 注意这里的 KERNEL_END_ADDRESS 为 ref 类型，需要加 *
    println!(
        "KERNEL_END_ADDRESS: {}",
        *memory::config::KERNEL_END_ADDRESS
    );

    // 物理页分配
    for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
        println!(
            "Frame address range: {} - {}",
            frame_0.address(),
            frame_1.address()
        );
    }
    println!("==> memory alloc test passed");
}

fn kernel_remap_test() {
    let remap = memory::mapping::MemorySet::new_kernel().unwrap();
    remap.activate();

    println!("==> kernel remapped test passed");
}
