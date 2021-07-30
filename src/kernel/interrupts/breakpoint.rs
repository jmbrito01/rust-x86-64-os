use x86_64::structures::idt::InterruptStackFrame;
use crate::kprintln;


pub extern "x86-interrupt" fn breakpoint_handler(
  stack_frame: InterruptStackFrame)
{
  kprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}