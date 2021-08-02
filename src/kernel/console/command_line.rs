
use alloc::{string::{String, ToString}, vec::Vec};
use crate::{kernel::{time}, kprintln};
use super::{console, task};


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
      time::sleep(ms);
    },
    "exec" => {
      let command = args_iter.next().unwrap();
      let args: Vec<String> = args_iter
        .map(|e| e.to_string())
        .collect();
      
      task::command_line::push_command(command, args);
    },
    _ => {
      kprintln!("ERROR: Invalid command: {}", command);
    }
  }

  console::print_input_prefix();
}