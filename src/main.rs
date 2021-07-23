#![no_std]
#![no_main]
#![feature(asm,abi_x86_interrupt)]

use core::panic::PanicInfo;

use bootloader::BootInfo;
use x86_64::{VirtAddr, structures::paging::{Translate}};

//use crate::memory::active_level_4_table;
bootloader::entry_point!(kernel_main);

mod vga_buffer;
mod interrupts;
mod gdt;
mod serial;
mod memory;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {
    x86_64::instructions::hlt();
  }
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

  let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
  // new: initialize a mapper
  let mapper = unsafe { memory::init(phys_mem_offset) };
  let addresses = [
    // the identity-mapped vga buffer page
    0xb8000,
    // some code page
    0x201008,
    // some stack page
    0x0100_0020_1a10,
    // virtual address mapped to physical address 0
    boot_info.physical_memory_offset,
  ];



  for &address in &addresses {
    let virt = VirtAddr::new(address);
    let phys = unsafe { mapper.translate_addr(virt) };
    println!("{:?} -> {:?}", virt, phys);
  }

  loop {
    x86_64::instructions::hlt();
  }
  // Simulate interrupt
  // x86_64::instructions::interrupts::int3(); // new

  // Simulate page fault
  // unsafe {
  //   *(0xdeadbeef as *mut u64) = 42;
  // };
  // fn stack_overflow() {
  //   stack_overflow(); // for each recursion, the return address is pushed
}