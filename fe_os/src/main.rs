#![no_std]
#![no_main]

mod vga_buffer;
use crate::vga_buffer::{Color, ColorCode}; 
use core::panic::PanicInfo;
use lazy_static::lazy_static;

lazy_static! {

    static ref RED_ON_BLACK:ColorCode = ColorCode::new(Color::LightRed, Color::Black);
    static ref BLACK_ON_RED:ColorCode = ColorCode::new(Color::Black, Color::LightRed);
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    
    vga_buffer::WRITER.lock().color_code = *BLACK_ON_RED;
    vga_buffer::WRITER.lock().write_string("PANIC!!\n");
    println!("{}",_info);
    loop {}
}

static HELLO : &str = "This is a test string, Hello x86_64!";


#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_buffer::WRITER.lock().write_string(HELLO);
    assert_eq!(0,1);
    loop {}
}
