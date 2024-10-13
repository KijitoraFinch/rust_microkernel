#![no_std]
#![no_main]
#![feature(naked_functions)]

mod sbi;
use common::println;
use common::Paddr;
use common::PAGE_SIZE;
use core::alloc;
use core::{
    arch::asm,
    option,
    panic::PanicInfo,
    ptr::{self, read},
};

macro_rules! readCsr {
    ($csr:expr) => {
        {
            let r: usize;
            unsafe {
                asm!(concat!("csrr {r},", $csr), r = out(reg) r);
            }
            r
        }
    };
}

macro_rules! writeCsr {
    ($csr:expr, $value:expr) => {
        unsafe {
            asm!(concat!("csrw ", $csr, ",{0}",), in(reg) $value);
        }
    };
}

extern "C" {
    static mut __bss: u32;
    static __bss_end: u32;
    static __stack_top: u32;
    static mut __free_ram: u8;
    static mut __free_ram_end: u8;
}

static mut NEXT_PADDR: *mut u8 = ptr::addr_of_mut!(__free_ram);
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

#[no_mangle]
#[naked]
extern "C" fn kernel_entry() -> ! {
    unsafe {
        // Let all registers saved on the stack
        asm!(
            "csrw sscratch, sp",
            "addi sp, sp, -4 * 31",
            "sw ra, 4 * 0(sp)",
            "sw gp, 4 * 1(sp)",
            "sw tp, 4 * 2(sp)",
            "sw t0, 4 * 3(sp)",
            "sw t1, 4 * 4(sp)",
            "sw t2, 4 * 5(sp)",
            "sw t3, 4 * 6(sp)",
            "sw t4, 4 * 7(sp)",
            "sw t5, 4 * 8(sp)",
            "sw t6, 4 * 9(sp)",
            "sw a0, 4 * 10(sp)",
            "sw a1, 4 * 11(sp)",
            "sw a2, 4 * 12(sp)",
            "sw a3, 4 * 13(sp)",
            "sw a4, 4 * 14(sp)",
            "sw a5, 4 * 15(sp)",
            "sw a6, 4 * 16(sp)",
            "sw a7, 4 * 17(sp)",
            "sw s0, 4 * 18(sp)",
            "sw s1, 4 * 19(sp)",
            "sw s2, 4 * 20(sp)",
            "sw s3, 4 * 21(sp)",
            "sw s4, 4 * 22(sp)",
            "sw s5, 4 * 23(sp)",
            "sw s6, 4 * 24(sp)",
            "sw s7, 4 * 25(sp)",
            "sw s8, 4 * 26(sp)",
            "sw s9, 4 * 27(sp)",
            "sw s10, 4 * 28(sp)",
            "sw s11, 4 * 29(sp)",
            "csrr a0, sscratch",
            "sw a0, 4 * 30(sp)",
            "mv a0, sp",
            "call handle_trap",
            // Restore all registers
            "lw ra, 4 * 0(sp)",
            "lw gp, 4 * 1(sp)",
            "lw tp, 4 * 2(sp)",
            "lw t0, 4 * 3(sp)",
            "lw t1, 4 * 4(sp)",
            "lw t2, 4 * 5(sp)",
            "lw t3, 4 * 6(sp)",
            "lw t4, 4 * 7(sp)",
            "lw t5, 4 * 8(sp)",
            "lw t6, 4 * 9(sp)",
            "lw a0, 4 * 10(sp)",
            "lw a1, 4 * 11(sp)",
            "lw a2, 4 * 12(sp)",
            "lw a3, 4 * 13(sp)",
            "lw a4, 4 * 14(sp)",
            "lw a5, 4 * 15(sp)",
            "lw a6, 4 * 16(sp)",
            "lw a7, 4 * 17(sp)",
            "lw s0, 4 * 18(sp)",
            "lw s1, 4 * 19(sp)",
            "lw s2, 4 * 20(sp)",
            "lw s3, 4 * 21(sp)",
            "lw s4, 4 * 22(sp)",
            "lw s5, 4 * 23(sp)",
            "lw s6, 4 * 24(sp)",
            "lw s7, 4 * 25(sp)",
            "lw s8, 4 * 26(sp)",
            "lw s9, 4 * 27(sp)",
            "lw s10, 4 * 28(sp)",
            "lw s11, 4 * 29(sp)",
            "lw sp, 4 * 30(sp)",
            "sret",
            options(noreturn)
        );
    }
}

#[repr(C, packed)]
struct TrapFrame {
    ra: u32,
    gp: u32,
    tp: u32,
    t0: u32,
    t1: u32,
    t2: u32,
    t3: u32,
    t4: u32,
    t5: u32,
    t6: u32,
    a0: u32,
    a1: u32,
    a2: u32,
    a3: u32,
    a4: u32,
    a5: u32,
    a6: u32,
    a7: u32,
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
    s4: u32,
    s5: u32,
    s6: u32,
    s7: u32,
    s8: u32,
    s9: u32,
    s10: u32,
    s11: u32,
    sp: u32,
}

#[no_mangle]
extern "C" fn handle_trap(trapframe: &TrapFrame) {
    let scause = readCsr!("scause");
    let stval = readCsr!("stval");
    let user_pc = readCsr!("sepc");
    panic!(
        "unexpected trap! scause={:x}, stval={:x}, user_pc={:x}",
        scause, stval, user_pc
    );
}

fn alloc_pages(pages: u32) -> Paddr {
    unsafe {
        let page_addr = NEXT_PADDR as Paddr;
        NEXT_PADDR = NEXT_PADDR.add((pages as usize * PAGE_SIZE));

        if NEXT_PADDR > ptr::addr_of_mut!(__free_ram_end) {
            panic!("out of memory");
        } else {
            ptr::write_bytes(page_addr as *mut u8, 0, pages as usize * PAGE_SIZE);
            page_addr as Paddr
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("Error: {:?}", _info);
    loop {}
}
