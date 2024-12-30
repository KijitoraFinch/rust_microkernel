#![no_std]
#![no_main]
#![feature(naked_functions)]

mod exception;
mod macros;
mod memory;
mod process;
mod sbi;

use common::println;
use common::Paddr;
use common::PAGE_SIZE;
use core::{arch::asm, panic::PanicInfo, ptr};
use exception::kernel_entry;
use memory::alloc_pages;
use sbi::putchar;

extern "C" {
    static mut __bss: u32;
    static __bss_end: u32;
    static __stack_top: u32;
    static mut __free_ram: u8;
    static mut __free_ram_end: u8;
}

static mut manager: process::ProcessManager = process::ProcessManager::new();

#[no_mangle]
fn procA() {
    println!("Process A started");
    let mut i = 0;
    loop {
        putchar(b'A' as u8);
        unsafe { manager.schedule() };
        for _ in 0..100000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

#[no_mangle]
fn procB() {
    println!("Process B started");
    let mut i = 0;
    loop {
        putchar(b'B' as u8);
        unsafe { manager.schedule() };
        for _ in 0..100000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}

#[no_mangle]
extern "C" fn kernel_main() -> ! {
    unsafe {
        let bss_start = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss_start, 0, bss_end as usize - bss_start as usize);
        let mut free_ram_start = ptr::addr_of_mut!(__free_ram);
        let free_ram_end = ptr::addr_of_mut!(__free_ram_end);
        ptr::write_bytes(
            free_ram_start,
            0,
            free_ram_end as usize - free_ram_start as usize,
        );
        writeCsr!("stvec", kernel_entry);
    }
    println!("Hello, world!");
    fn idle_dummy() {
        unsafe { asm!("nop") }
    }
    // register idle process
    unsafe {
        manager.current_process = process::CurrentProcess::Idle;
        // create idle process
        manager.create_idle(idle_dummy as usize);
        manager.create(procA as usize);
        manager.create(procB as usize);
        manager.schedule();
    }

    unreachable!("Kernel main should never return");
}

#[link_section = ".text.boot"]
#[naked]
#[no_mangle]
extern "C" fn boot() {
    unsafe {
        asm!("la sp, {stack_top}",
            "j kernel_main", 
            stack_top = sym __stack_top,
            options(noreturn));
    };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Error: {:?}", _info);
    loop {}
}
