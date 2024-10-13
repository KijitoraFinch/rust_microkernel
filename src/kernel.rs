#![no_std]
#![no_main]
#![feature(naked_functions)]

mod exception;
mod macros;
mod memory;
mod sbi;

use common::println;
use common::Paddr;
use common::PAGE_SIZE;
use core::{arch::asm, panic::PanicInfo, ptr};
use exception::kernel_entry;
use memory::alloc_pages;

extern "C" {
    static mut __bss: u32;
    static __bss_end: u32;
    static __stack_top: u32;
    static mut __free_ram: u8;
    static mut __free_ram_end: u8;
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

    unsafe {
        let paddr = alloc_pages(1);
        println!("Allocated page at 0x{:x}", paddr);
        let paddr1 = alloc_pages(1);
        println!("Allocated page at 0x{:x}", paddr1);
        assert!(paddr + PAGE_SIZE == paddr1);
    }
    loop {}
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
