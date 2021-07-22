#![no_std]
#![no_main]
#![feature(asm,abi_x86_interrupt)]

use core::panic::PanicInfo;

use bootloader::BootInfo;
bootloader::entry_point!(kernel_main);

mod vga_buffer;
mod interrupts;
mod gdt;
mod serial;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {
    x86_64::instructions::hlt();
  }
}

#[no_mangle]
fn kernel_main(boot_info:   &'static BootInfo) -> ! {
  // This is where the kernel code starts
  println!("[ KERNEL START ] Started loading kernel...");

  // Initiate GDT
  gdt::init();

  // Initiate interrupt handlers
  interrupts::init_idt();
  unsafe { interrupts::PICS.lock().initialize() };
  x86_64::instructions::interrupts::enable();

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