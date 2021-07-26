use x86_64::structures::idt::InterruptStackFrame;
use lazy_static::lazy_static;
use crate::interrupts::{InterruptIndex, PICS};

pub extern "x86-interrupt" fn keyboard_interrupt_handler(
  _stack_frame: InterruptStackFrame
) {
  use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
  use spin::Mutex;
  use x86_64::instructions::port::Port;

  lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
      Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
        HandleControl::Ignore)
      );
  }

  let mut port = Port::new(0x60);

  let scancode: u8 = unsafe { port.read() };
  crate::task::keyboard::add_scancode(scancode); // new

  unsafe {
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
  }
}