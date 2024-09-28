#![no_std]
#![no_main]

use core::fmt::Write;

extern "C" {
    fn putchar(c: u8);
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            for c in s.as_bytes(){
                putchar(*c);
            }
        }
        core::fmt::Result::Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    let mut con = Console{};
    con.write_fmt(args).unwrap()
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}