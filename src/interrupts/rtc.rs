use x86_64::structures::idt::InterruptStackFrame;
use crate::interrupts::{InterruptIndex, PICS};
use crate::{println, time};

pub extern "x86-interrupt" fn rtc_interrupt_handler(
  stack_frame: InterruptStackFrame)
{
  println!(".");
  time::on_rtc_interrupt(stack_frame);
  
  unsafe {
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::RTC.as_u8());
  }
}