#![no_std]
#![no_main]
#![feature(naked_functions)]

mod sbi;
use core::{arch::asm, panic::PanicInfo, ptr};
use common::println;

extern "C" {
    static mut __bss: u32;
    static __bss_end: u32;
    static __stack_top: u32;
}

#[no_mangle]
extern "C" fn kernel_main() -> ! {
    unsafe {
        let bss_start = ptr::addr_of_mut!(__bss);
        let bss_end = ptr::addr_of!(__bss_end);
        ptr::write_bytes(bss_start, 0, bss_end as usize - bss_start as usize);
    }
    println!("Hello, world!");
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
    loop{}
}