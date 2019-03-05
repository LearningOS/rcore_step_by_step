use bbl::sbi;
use alloc::string::String;
use core::fmt::{self, Write};

pub fn putchar(ch: usize) {
    sbi::console_putchar(ch);
}

pub fn puts(s: &str) {
    for &byte in s.as_bytes() {
        putchar(byte as usize);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::io::_print(format_args!($($arg)*));
    });
}

pub fn _print(args: fmt::Arguments) {
    StdOut.write_fmt(args).unwrap();
}

struct StdOut;

impl fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        puts(s);
        Ok(())
    }
}
