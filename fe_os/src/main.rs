#![no_std]
#![no_main]

mod vga_buffer;
use crate::vga_buffer::{Color, ColorCode}; 
use core::panic::PanicInfo;
use lazy_static::lazy_static;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO : &str = "This is a test string, Hello x86_64!";

lazy_static! {
    static ref redBlue:ColorCode = ColorCode::new(Color::Red, Color::Blue);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_buffer::WRITER.lock().write_string_color(HELLO, *redBlue);
    loop {}
}
