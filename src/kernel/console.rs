use core::cmp::Ordering;

use crate::{kernel::command_line, kprint, kprintln};
use alloc::{string::{String, ToString}, vec::Vec};
use lazy_static::lazy_static;
use pc_keyboard::{KeyCode};
use spin::Mutex;

use super::vga_buffer;

lazy_static! {
  static ref STDIN: Mutex<String> = Mutex::new(String::new());
  static ref WAITING_INPUT: Mutex<bool> = Mutex::new(false);
}

pub fn init() {
  kprintln!("[ CONSOLE ] Starting to load console...");

  set_waiting_input(true);
}

pub fn is_waiting_input() -> bool {
  *WAITING_INPUT.lock()
}

fn set_waiting_input(value: bool) -> bool {
  let mut waiting_input = WAITING_INPUT.lock();
  let was_enabled = waiting_input.clone();
  *waiting_input = value;

  if value == true && was_enabled == false {
    // Start a new command line
    kprint!("> ");
  }

  was_enabled
}

fn on_read_line(line: &str) {
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

  command_line::run_command(command, args);
}

fn without_input<F, R>(f: F) -> R where F: FnOnce() -> R {
  let was_enabled = set_waiting_input(false);
  
  let ret = f();

  if was_enabled {
    set_waiting_input(true);
  }

  ret
}

pub fn handle_unicode_press(c: char) {
  if is_waiting_input() {
    if c.cmp(&'\n') == Ordering::Equal { // ENTER
      without_input(|| {
        set_waiting_input(false);
        let mut str = STDIN.lock();
        on_read_line(&str);
        str.clear();
      });
    } else if c.cmp(&'\x7F') == Ordering::Equal || c.cmp(&'\x08') == Ordering::Equal { // BACKSPACE
      // Erase last caracter
      let mut str = STDIN.lock();
      if str.len() > 0 {
        str.pop();
        vga_buffer::erase_last();
      }
    } else {
      // Handle Alphabet characters
      let mut str = STDIN.lock();
      str.push(c);
      kprint!("{}", c);
    }
    
  }
}

pub fn handle_raw_press(key: KeyCode) {
  match key {
    _ => {
      //kprint!("{:?}", key)
    },
  }
}