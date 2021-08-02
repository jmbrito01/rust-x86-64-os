#![no_std]
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
  // Initiate clock operations
  kernel::time::init();

  // Initiate GDT
  kernel::gdt::init();

  // Initiate interrupt handlers
  kernel::interrupts::init();

  // Initialize Memory and Heap
  unsafe {
    kernel::memory::init(boot_info);
  }

  // Initiate PCI Controllers
  kernel::pci::init();

  kernel::console::init(true); // Init console with user inputs

  // Start running kernel tasks
  let mut kernel_thread = kernel::task::kernel_worker();

  kernel_thread.run()
}