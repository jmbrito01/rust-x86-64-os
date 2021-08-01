use alloc::{string::String, vec::Vec};

use crate::kprintln;

use super::interrupts;

pub fn run_command(command: &str, args: Vec<String>) {
  interrupts::without_interrupts(|| {
    match command {
      "echo" => {
        kprintln!("{}", args.join(" "));
      },
      "curtime" => {
        use crate::kernel::cmos::CMOS;
        let rtc = CMOS::new().rtc();
        kprintln!("{}/{}/{} - {}:{}:{}", rtc.day, rtc.month, rtc.year, rtc.hour, rtc.minute, rtc.second);
      }
      _ => {
        kprintln!("ERROR: Invalid command: {}", command);
      },
    }
  });
}