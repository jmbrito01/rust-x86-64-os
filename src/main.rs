#![no_std]
#![no_main]
#![feature(asm,llvm_asm,abi_x86_interrupt,alloc_error_handler)]

use core::panic::PanicInfo;

use bootloader::BootInfo;
use os_x86::kprintln;

//use crate::memory::active_level_4_table;
bootloader::entry_point!(kernel_main);

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {

  loop {
    x86_64::instructions::hlt();
  }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
  // This is where the kernel code starts
  kprintln!("[ KERNEL START ] Started loading kernel...");

  os_x86::init(boot_info)
}