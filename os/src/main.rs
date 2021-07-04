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

#[macro_use]
mod console;
mod panic;
mod sbi;
mod interrupt;
mod memory;

extern crate alloc;

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

/// Rust 的入口函数
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> !{
    println!("Boot success!\n");
    println!("Hello rCore-Tutorial!");

    // 初始化各种模块
    interrupt::init();
    memory::init();

    // interrupt_test();
    dynamic_memory_alloc_test();
    physical_memory_alloc_test();

    unreachable!();
    // loop{}
}

fn interrupt_test(){
    // 中断测试
    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    };
}

fn dynamic_memory_alloc_test(){
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

fn physical_memory_alloc_test(){
    // 注意这里的 KERNEL_END_ADDRESS 为 ref 类型，需要加 *
    println!("KERNEL_END_ADDRESS: {}", *memory::config::KERNEL_END_ADDRESS);

    // 物理页分配
    for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        println!("Frame address range: {} - {}", frame_0.address(), frame_1.address());
    }
    println!("==> memory alloc test passed");
}