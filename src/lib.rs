#![no_std]
#![no_main]
#![feature(asm,llvm_asm,abi_x86_interrupt,alloc_error_handler)]

use bootloader::BootInfo;

#[macro_use]
pub mod kernel;


extern crate alloc;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn init(boot_info: &'static BootInfo) -> ! {
  // Initiate GDT
  kernel::gdt::init();

  // Initiate interrupt handlers
  kernel::interrupts::init_idt();
  unsafe { kernel::interrupts::PICS.lock().initialize() };
  x86_64::instructions::interrupts::enable();

  // Initiate clock operations
  kernel::time::init();
  let rtc = kernel::cmos::CMOS::new().rtc();
  kprintln!("Current Time: {}/{}/{} - {}:{}:{}", rtc.day, rtc.month, rtc.year, rtc.hour, rtc.minute, rtc.second);

  // Initializee Memory and Heap
  unsafe {
    kernel::memory::init(boot_info);
  }

  let mut thread = kernel::task::executor::Executor::new();
  thread.spawn(kernel::task::Task::new(kernel::task::keyboard::print_keypresses()));

  thread.run();
}