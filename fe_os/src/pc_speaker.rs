use x86_64::instructions::port::Port;
use crate::interrupts;

const SPK_CONFIG_PORT: u16 = 0x61;

pub fn connect() {
    let mut spk_conf = Port::<u8>::new(SPK_CONFIG_PORT);
    let mut value = unsafe { spk_conf.read() };
    value = value | 0b00000011;
    unsafe { spk_conf.write(value); }
}

pub fn disconnect() {
    let mut spk_conf = Port::<u8>::new(SPK_CONFIG_PORT);
    let mut value = unsafe { spk_conf.read() };
    value = value & 0b11111100;
    unsafe { spk_conf.write(value); }
}


pub fn play_freq(freq: u32) {
    let f = interrupts::Frequency::from_freq(freq);
    interrupts::timer2_write_freq(f);
}

