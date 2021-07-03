//! 内存模块
//! 
//! 

mod config;
mod heap;

pub fn init() {
    heap::init();
    println!("mod memory initialized");
}