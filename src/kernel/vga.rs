use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use super::interrupts;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGray = 7,
  DarkGray = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14,
  White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
  fn new(foreground: Color, background: Color) -> ColorCode {
    ColorCode((background as u8) << 4 | (foreground as u8))
  }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
  ascii_character: u8,
  color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
  column_position: usize,
  color_code: ColorCode,
  buffer: &'static mut Buffer,
}

impl Writer {
  pub fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => {
        // Clear cursor before going to next line
        self.erase_cursor();
        self.new_line();
      },
      byte => {
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        let color_code = self.color_code;
        self.buffer.chars[row][col].write(ScreenChar {
          ascii_character: byte,
          color_code,
        });
        self.column_position += 1;

        // Print cursor
        self.show_cursor();
      }
    }
  }

  fn erase_cursor(&mut self) {
    let blank = ScreenChar {
      ascii_character: b' ',
      color_code: self.color_code,
    };
    let row = BUFFER_HEIGHT - 1;
    self.buffer.chars[row][self.column_position].write(blank);
  }

  fn show_cursor(&mut self) {
    let cursor = ScreenChar {
      ascii_character: b' ',
      color_code: ColorCode::new(Color::Yellow, Color::Yellow),
    };
    let row = BUFFER_HEIGHT - 1;
    self.buffer.chars[row][self.column_position].write(cursor);
  } 

  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let character = self.buffer.chars[row][col].read();
        self.buffer.chars[row - 1][col].write(character);
      }
    }
    self.clear_row(BUFFER_HEIGHT - 1);
    self.column_position = 0;
  }

  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_character: b' ',
      color_code: self.color_code,
    };
    for col in 0..BUFFER_WIDTH {
      self.buffer.chars[row][col].write(blank);
    }
  }

  pub fn erase_last_char(&mut self, size: usize) {
    let blank = ScreenChar {
      ascii_character: b' ',
      color_code: self.color_code,
    };
    let row = BUFFER_HEIGHT - 1;
    let mut columns_to_erase = size.clone();
    if columns_to_erase > self.column_position {
      columns_to_erase = self.column_position
    }
    let pos_erased_cur_column = self.column_position - columns_to_erase;
    self.erase_cursor();

    for pos in self.column_position..pos_erased_cur_column {
      self.buffer.chars[row][pos].write(blank);
    }

    self.column_position -= columns_to_erase;
    self.show_cursor();
  }

  pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      match byte {
        // printable ASCII byte or newline
        0x20..=0x7e | b'\n' => self.write_byte(byte),
        // not part of printable ASCII range
        _ => self.write_byte(0xfe),
      }
    }
  }
}

impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write_string(s);
    Ok(())
  }
}

lazy_static! {
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  });
}

#[macro_export]
macro_rules! kprint {
  ($($arg:tt)*) => ($crate::kernel::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
  () => ($crate::print!("\n"));
  ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
  use core::fmt::Write;
  use x86_64::instructions::interrupts;  
  
  // Avoid deadlocks on writer when writing in parallel
  interrupts::without_interrupts(|| { 
    WRITER.lock().write_fmt(args).unwrap();
  });
}

pub fn erase_last_character(size: usize) {
  interrupts::without_interrupts(|| {
    WRITER.lock().erase_last_char(size);
  });
}