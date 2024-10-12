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
            for c in s.as_bytes() {
                putchar(*c);
            }
        }
        core::fmt::Result::Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    let mut con = Console {};
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

// Alternatives to the C standard library

pub fn strlen(s: &str) -> usize {
    let mut len = 0;
    for _ in s.chars() {
        len += 1;
    }
    len
}


pub fn strcpy(dst: &mut [u8], src: &str) {
    if dst.len() < src.len() {
        panic!("common/strcpy: destination buffer is too small");
    }    
    for (i, c) in src.bytes().enumerate() {
        dst[i] = c;
    }
}
