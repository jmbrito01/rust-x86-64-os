use alloc::{string::String, vec::Vec};

use crate::{kernel::time, kprintln};

use super::{console, interrupts};

pub async fn run_command(command: &str, args: Vec<String>) {
  let mut args_iter = args.iter();
  match command {
    "echo" => {
      kprintln!("{}", args.join(" "));
    },
    "curtime" => {
      use crate::kernel::cmos::CMOS;
      let rtc = CMOS::new().rtc();
      kprintln!("{}/{}/{} - {}:{}:{}", rtc.day, rtc.month, rtc.year, rtc.hour, rtc.minute, rtc.second);
    },
    "sleep" => {
      let seconds_string = args_iter.next()
        .expect("Sleep command requires one parameter");
      let ms = seconds_string.parse::<f64>()
        .expect("Invalid argument seconds");
      kprintln!("Sleeping for {} seconds", ms);
      time::sleep(ms);
    }
    _ => {
      kprintln!("ERROR: Invalid command: {}", command);
    }
  }

  console::print_input_prefix();
}