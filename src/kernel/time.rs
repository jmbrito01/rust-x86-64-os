use core::{convert::TryInto, sync::atomic::{AtomicUsize, AtomicU64, Ordering}};
use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};
use crate::{kprintln, kernel::{interrupts}};

const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0; // 1_193_181.666 Hz
const PIT_DIVIDER: usize = 1193;
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

const PIT_COMMAND_PORT: u16 = 0x43;
const PIT_DATA_PORT: u16 = 0x40;

pub static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);
static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);
static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);

fn set_pit_frequency_divider(divider: u16) {
  interrupts::without_interrupts(|| {
    let bytes = divider.to_le_bytes();
    let mut cmd: Port<u8> = Port::new(PIT_COMMAND_PORT);
    let mut data: Port<u8> = Port::new(PIT_DATA_PORT);

    unsafe {
      cmd.write(0x36);
      data.write(bytes[0]);
      data.write(bytes[1]);
    }
  });
}

fn rdtsc() -> u64 {
  let upper: u64;
let lower: u64;
unsafe {
    llvm_asm!("rdtsc"
         : "={rax}"(lower), 
           "={rdx}"(upper)
         :
         :
         : "volatile"
    );
}
upper << 32 | lower
}

pub fn time_between_ticks() -> f64 {
  PIT_INTERVAL
}

pub fn uptime() -> f64 {
  time_between_ticks() * ticks() as f64
}

pub fn last_rtc_update() -> usize {
  LAST_RTC_UPDATE.load(Ordering::Relaxed)
}

pub fn sleep(seconds: f64) {
  let start = uptime();
  while uptime() - start < seconds {
    x86_64::instructions::hlt();
  }
}

pub fn ticks() -> usize {
  PIT_TICKS.load(Ordering::Relaxed)
}

pub fn nanowait(nanosecs: u64) {
  let start = rdtsc();
  let delta = nanosecs * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);
  while rdtsc() - start < delta {
    core::hint::spin_loop();
  }
}

pub fn on_timer_interrupt(
  _stack_frame: InterruptStackFrame
) {
  PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn on_rtc_interrupt(
  _stack_frame: InterruptStackFrame
) {
  LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
}

pub fn init() {
  // PIT Timer
  // At boot the PIT starts with a frequency divider of 0 (equivalent to 65536)
  // During init we will change the divider to 1193 to have about 1.000 ms
  // between ticks to improve time measurements accuracy.

  let divider = if PIT_DIVIDER < 65536 { PIT_DIVIDER } else { 0 };
  set_pit_frequency_divider(divider.try_into().unwrap());

}