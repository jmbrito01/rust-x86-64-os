use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use crate::kprintln;

pub extern "x86-interrupt" fn page_fault_handler(
  stack_frame: InterruptStackFrame,
  error_code: PageFaultErrorCode,
) {
  use x86_64::registers::control::Cr2;

  kprintln!("EXCEPTION: PAGE FAULT");
  kprintln!("Accessed Address: {:?}", Cr2::read());
  kprintln!("Error Code: {:?}", error_code);
  kprintln!("{:#?}", stack_frame);
}