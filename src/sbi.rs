use core::arch::asm;

pub struct Sbiret {
    pub error: i32,
    pub value: i32,
}

pub unsafe fn sbi_call(
    a0: i32,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
    fid: i32,
    eid: i32,
) -> Sbiret {
    let error: i32;
    let value: i32;
    asm!(
        "ecall",
        inout("a0") a0 => error,
        inout("a1") a1 => value,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
        in("a5") a5,
        in("a6") fid,
        in("a7") eid,
    );
    Sbiret { error, value }
}

#[no_mangle]
pub fn putchar(c: u8) {
    unsafe {
        sbi_call(c as i32, 0, 0, 0, 0, 0, 0, 1);
    }
}

