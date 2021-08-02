use core::fmt::Write;

use alloc::{string::{String, ToString}, vec::Vec};
use lazy_static::lazy_static;
use pc_keyboard::{KeyCode};
use spin::Mutex;

use crate::{kernel::{console, task}, kprint, kprintln};

use super::vga;

pub mod command_line;

lazy_static! {
  static ref STDIN: Mutex<String> = Mutex::new(String::new());
  static ref WAITING_INPUT: Mutex<bool> = Mutex::new(false);
  static ref INPUT_HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());
  static ref INPUT_HISTORY_INDEX: Mutex<usize> = Mutex::new(0);
}

pub fn init(enable_input: bool) {
  kprintln!("[ CONSOLE ] Starting to load console...");

  if enable_input {
    set_waiting_input(true);
  }
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
  INPUT_HISTORY.lock().push(String::from(line));

  kprint!("\n"); // Go to the next line
  let mut iter = line.split(" ");
  let command = iter.next().unwrap();

  if command.is_empty() {
    kprintln!("Empty command is not allowed.");
    console::print_input_prefix();
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
        vga::erase_last_character(1);
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
    KeyCode::ArrowUp => {
      set_history_input(SetHistoryDirection::UP)
    },
    KeyCode::ArrowDown => {
      set_history_input(SetHistoryDirection::DOWN)
    },
    _ => kprint!("{:?}", key),
  }
}

enum SetHistoryDirection {
  UP,
  DOWN
}

fn set_history_input(direction: SetHistoryDirection) {
  let history = INPUT_HISTORY.lock();
  if history.len() == 0 {
    return;
  }
  let mut history_index = INPUT_HISTORY_INDEX.lock();

  

  let mut index = *history_index;
  match direction {
    SetHistoryDirection::DOWN => {
      if *history_index <= 1 {
        return;
      }
      index -= 1
    },
    SetHistoryDirection::UP => {
      if *history_index == history.len()  {
        return;
      }
      index += 1
    },
  }

  let item = history.get(history.len() - index)
    .unwrap();
  let mut stdin = STDIN.lock();
  vga::erase_last_character(stdin.len());
  stdin.clear();
  stdin.write_str(item)
    .unwrap();
  *history_index = index;
  kprint!("{}", item);
}