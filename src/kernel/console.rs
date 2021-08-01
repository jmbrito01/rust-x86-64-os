use crate::{kernel::{task}, kprint, kprintln};
use alloc::{string::{String, ToString}, vec::Vec};
use lazy_static::lazy_static;
use pc_keyboard::{KeyCode};
use spin::Mutex;

use super::{vga_buffer};

lazy_static! {
  static ref STDIN: Mutex<String> = Mutex::new(String::new());
  static ref WAITING_INPUT: Mutex<bool> = Mutex::new(false);
}

pub fn init() {
  kprintln!("[ CONSOLE ] Starting to load console...");
}

pub fn is_waiting_input() -> bool {
  *WAITING_INPUT.lock()
}

pub fn set_waiting_input(value: bool) -> bool {
  let mut waiting_input = WAITING_INPUT.lock();
  let was_enabled = waiting_input.clone();
  *waiting_input = value;

  if value == true && was_enabled == false {
    // Start a new command line
    print_input_prefix();
  }

  was_enabled
}

pub fn print_input_prefix() {
  kprint!("> ");
}

async fn on_read_line(line: &str) {
  kprint!("\n"); // Go to the next line
  let mut iter = line.split(" ");
  let command = iter.next().unwrap();

  if command.is_empty() {
    kprintln!("Empty command is not allowed.");
    return;
  }

  let args: Vec<String> = iter
    .map(|e| e.to_string())
    .collect();

  kprintln!("Command: {} / Arguments: {:?}", command, args);

  task::command_line::push_command(command, args);
}

pub async fn handle_unicode_press(c: char) {
  if is_waiting_input() == false {
  }
  // TODO: Do this enforcing no input
  match c {
    '\n' => { // ENTER
      let mut str = STDIN.lock();
      on_read_line(&str).await;
      str.clear();
    },
    '\x7F' | '\x08' => { // BACKSPACE
      // Erase last caracter
      let mut str = STDIN.lock();
      if str.len() > 0 {
        str.pop();
        vga_buffer::erase_last();
      }
    },
    _ => {
      let mut str = STDIN.lock();
      str.push(c);
      kprint!("{}", c);
    },
  }
}

pub async fn handle_raw_press(key: KeyCode) {
  match key {
    _ => {
      //kprint!("{:?}", key)
    },
  }
}