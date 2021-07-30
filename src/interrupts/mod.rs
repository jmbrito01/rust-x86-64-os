

use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use lazy_static::lazy_static;
use x86_64::structures::idt::PageFaultErrorCode;
use crate::gdt;
use crate::println;
use pic8259::ChainedPics;
use spin;

pub mod keyboard;
pub mod timer;
pub mod page_fault;
pub mod breakpoint;
pub mod double_fault;

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

    idt
  };
}

pub fn init_idt() {
  println!("[ INTERRUPTS ] Starting to load IDT.");
  IDT.load();
  println!("[ INTERRUPTS ] IDT loaded successfully.");
}



#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    FloppyDisk = 6,
    PrimaryATAHardDisk = 14,
    SecondaryATAHardDisk = 15,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}