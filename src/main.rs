#![no_std]
#![no_main]
#![feature(asm,abi_x86_interrupt,alloc_error_handler)]

use core::panic::PanicInfo;

use bootloader::BootInfo;
use x86_64::{VirtAddr, structures::paging::{Translate}};

use crate::memory::BootInfoFrameAllocator;
extern crate alloc;

//use crate::memory::active_level_4_table;
bootloader::entry_point!(kernel_main);

mod vga_buffer;
mod interrupts;
mod gdt;
mod serial;
mod memory;
mod task;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {
    x86_64::instructions::hlt();
  }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
  // This is where the kernel code starts
  println!("[ KERNEL START ] Started loading kernel...");

  // Initiate GDT
  gdt::init();

  // Initiate interrupt handlers
  interrupts::init_idt();
  unsafe { interrupts::PICS.lock().initialize() };
  x86_64::instructions::interrupts::enable();

  // Initializee Page Tables and Heap
  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  let mut mapper = unsafe { memory::init(phys_mem_offset) };
  let mut frame_allocator = unsafe {
    BootInfoFrameAllocator::init(&boot_info.memory_map)
  };
  memory::allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("Heap Allocation failed");

  let mut thread = task::executor::Executor::new();
  thread.spawn(task::Task::new(task::keyboard::print_keypresses()));

  thread.run();
}

fn get_last_result() {

}