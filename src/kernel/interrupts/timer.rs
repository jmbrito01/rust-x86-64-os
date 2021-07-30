use x86_64::structures::idt::InterruptStackFrame;
use crate::kernel::interrupts::{InterruptIndex, PICS};
use crate::kernel::time;

pub extern "x86-interrupt" fn timer_interrupt_handler(
  stack_frame: InterruptStackFrame)
{
  time::on_timer_interrupt(stack_frame);

  unsafe {
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  }
}