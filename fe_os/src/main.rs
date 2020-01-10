#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(fe_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use fe_os::{println, print};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;


entry_point!(kernel_main);

use pc_keyboard::DecodedKey; 

mod pc_speaker;

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static!{
    static ref FREQ: Mutex<u32> = Mutex::new(440);
}

fn keypress_handler(key: DecodedKey) {
    print!("{:?}", key);
    *(FREQ.lock()) += 50;
    pc_speaker::play_freq(*(FREQ.lock()));
    match key {
        DecodedKey::Unicode('a') => { pc_speaker::connect(); print!("ON"); },
        _ => { pc_speaker::disconnect(); print!("OFF"); },
    }
}

use fe_os::interrupts;

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use fe_os::allocator;
    use fe_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    fe_os::init();


    unsafe {
        interrupts::KEYPRESS_HANDLER = keypress_handler;
    }

    interrupts::timer0_write_freq(interrupts::Frequency::from_freq(1000));

    println!("Boot info: {:?}", boot_info);

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );


    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    fe_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    fe_os::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    fe_os::test_panic_handler(info)
}
