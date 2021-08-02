

use x86_64::structures::idt::InterruptDescriptorTable;
use lazy_static::lazy_static;
use crate::kernel::gdt;

use pic8259::ChainedPics;
use spin;

pub mod keyboard;
pub mod timer;
pub mod page_fault;
pub mod breakpoint;
pub mod double_fault;
pub mod rtc;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
  spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint::breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault::page_fault_handler);
    unsafe {
      idt.double_fault
        .set_handler_fn(double_fault::double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); 
    };
    idt[InterruptIndex::Timer.as_usize()]
      .set_handler_fn(timer::timer_interrupt_handler);

    idt[InterruptIndex::Keyboard.as_usize()]
      .set_handler_fn(keyboard::keyboard_interrupt_handler);

    idt[InterruptIndex::RTC.as_usize()]
      .set_handler_fn(rtc::rtc_interrupt_handler);

    idt
  };
}

pub fn init() {
  IDT.load();
  
  unsafe { PICS.lock().initialize() };
  x86_64::instructions::interrupts::enable();
}

pub fn without_interrupts<F, R>(f: F) -> R where F: FnOnce() -> R {
  use x86_64::instructions::interrupts::{are_enabled, disable, enable};

  let is_enabled = are_enabled();

  if is_enabled {
    disable();
  }

  let ret = f();

  // Only re-enable if it was enabled before
  if is_enabled {
    enable();
  }

  ret
}


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
    FloppyDisk = PIC_1_OFFSET + 6,
    RTC = PIC_1_OFFSET + 8,
    PrimaryATAHardDisk = PIC_1_OFFSET + 14,
    SecondaryATAHardDisk = PIC_1_OFFSET + 15,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}