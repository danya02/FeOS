use x86_64::instructions::port::{Port, PortWriteOnly};

const PC_SPEAKER_CONFIG_PORT: u16 = 0x61;
const TIMER2_COUNTDOWN_BEGIN_PORT: u16 = 0x43;
const TIMER2_COUNTDOWN_BEGIN_VALUE: u8 = 0xb6;
const TIMER2_COUNTDOWN_VALUE_PORT: u16 = 0x42;


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

pub fn connect() {
    let mut pc_speaker_conf = Port::<u8>::new(PC_SPEAKER_CONFIG_PORT);
    let mut value = unsafe { pc_speaker_conf.read() };
    value = value | 0b00000011;
    unsafe { pc_speaker_conf.write(value); }
}

pub fn disconnect() {
    let mut pc_speaker_conf = Port::<u8>::new(PC_SPEAKER_CONFIG_PORT);
    let mut value = unsafe { pc_speaker_conf.read() };
    value = value & 0b11111100;
    unsafe { pc_speaker_conf.write(value); }
}

pub fn write_freq(val: Frequency) {
    let mut freq_conf_begin = PortWriteOnly::<u8>::new(TIMER2_COUNTDOWN_BEGIN_PORT);
    let mut freq_conf_val = PortWriteOnly::<u8>::new(TIMER2_COUNTDOWN_VALUE_PORT);
    unsafe {
        freq_conf_begin.write(TIMER2_COUNTDOWN_BEGIN_VALUE);
        freq_conf_val.write(val.low);
        freq_conf_val.write(val.high);
    }
}

