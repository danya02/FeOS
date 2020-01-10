// The x86-interrupt calling convention leads to the following LLVM error
// when compiled for a Windows target: "offset is not a multiple of 16". This
// happens for example when running `cargo test` on Windows. To avoid this
// problem we skip compilation of this module on Windows.
#![cfg(not(windows))]

use crate::{gdt, hlt_loop, print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use x86_64::instructions::port::PortWriteOnly;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

pub static mut TIMER_HANDLER: fn() -> () = | | {};

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    print!(".");
    unsafe{ TIMER_HANDLER(); }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}


const TIMER_CMD_PORT: u16 = 0x43;
const TIMER_CH0_PORT: u16 = 0x40;
const TIMER_CH2_PORT: u16 = 0x42;

const TIMER_CH0_SET_CMD: u8 = 0b00110110;
const TIMER_CH2_SET_CMD: u8 = 0b10110110;

pub struct Frequency {
    high: u8,
    low: u8,
}

impl Frequency {
    pub fn from_countdown(val: u16) -> Frequency {
        let hi = (val / 256) as u8;
        let lo = (val % 256) as u8;
        Frequency {high: hi, low: lo}
    }

    pub fn from_freq(val: u32) -> Frequency {
        let countdown = (1193180 / val) as u16;
        Frequency::from_countdown(countdown)
    }
}

pub fn timer0_write_freq(val: Frequency) {
    let mut freq_conf_begin = PortWriteOnly::<u8>::new(TIMER_CMD_PORT);
    let mut freq_conf_val = PortWriteOnly::<u8>::new(TIMER_CH0_PORT);
    unsafe {
        freq_conf_begin.write(TIMER_CH0_SET_CMD);
        freq_conf_val.write(val.low);
        freq_conf_val.write(val.high);
    }
}

pub fn timer2_write_freq(val: Frequency) {
    let mut freq_conf_begin = PortWriteOnly::<u8>::new(TIMER_CMD_PORT);
    let mut freq_conf_val = PortWriteOnly::<u8>::new(TIMER_CH2_PORT);
    unsafe {
        freq_conf_begin.write(TIMER_CH2_SET_CMD);
        freq_conf_val.write(val.low);
        freq_conf_val.write(val.high);
    }
}


use pc_keyboard::DecodedKey;
pub static mut KEYPRESS_HANDLER: fn(DecodedKey) -> () = |_x: DecodedKey| {}; 

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            unsafe {KEYPRESS_HANDLER(key);}
            //match key {
            //    DecodedKey::Unicode(character) => print!("{}", character),
            //    DecodedKey::RawKey(key) => print!("{:?}", key),
            //}
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception...");
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}
