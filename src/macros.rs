#[macro_export]
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
#[macro_export]
macro_rules! writeCsr {
    ($csr:expr, $value:expr) => {
        unsafe {
            asm!(concat!("csrw ", $csr, ",{0}",), in(reg) $value);
        }
    };
}
