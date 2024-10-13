use common::Paddr;
use common::PAGE_SIZE;
use core::ptr;

extern "C" {
    static mut __free_ram: u8;
    static mut __free_ram_end: u8;
}

static mut NEXT_PADDR: *mut u8 = ptr::addr_of_mut!(__free_ram);

pub fn alloc_pages(pages: u32) -> Paddr {
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
