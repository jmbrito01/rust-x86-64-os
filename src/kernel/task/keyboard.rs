use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1, layouts};
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;

use crate::{kernel::{self}, kprintln};  

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
  if let Ok(queue) = SCANCODE_QUEUE.try_get() {
    if let Err(_) = queue.push(scancode) {
      kprintln!("WARNING: scancode queue full; dropping keyboard input");
    } else {
      WAKER.wake();
    }
  } else {
    kprintln!("WARNING: scancode queue uninitialized");
  }
}

async fn handle_unicode_press(c: char) {
  // TODO: Do this without interrupts
  kernel::console::handle_unicode_press(c).await
}

async fn handle_raw_press(c: KeyCode) {
  // TODO: Do this without interrupts
  kernel::console::handle_raw_press(c).await
}

pub async fn handle_keypresses() {
  let mut scancodes = ScancodeStream::new();
  let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
  while let Some(scancode) = scancodes.next().await {
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
      if let Some(key) = keyboard.process_keyevent(key_event) {
        match key {
          DecodedKey::Unicode(character) => handle_unicode_press(character).await,
          DecodedKey::RawKey(key) => handle_raw_press(key).await,
        }
      }
    }
  }
}

pub struct ScancodeStream {
  _private: (),
}

impl ScancodeStream {
  pub fn new() -> Self {
    SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
      .expect("ScancodeStream::new should only be called once");
    ScancodeStream { _private: () }
  }
}

impl Stream for ScancodeStream {
  type Item = u8;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
    let queue = SCANCODE_QUEUE
      .try_get()
      .expect("scancode queue not initialized");
    // fast path
    if let Ok(scancode) = queue.pop() {
        return Poll::Ready(Some(scancode));
    }

    WAKER.register(&cx.waker());
    match queue.pop() {
      Ok(scancode) => {
        WAKER.take();
        Poll::Ready(Some(scancode))
      }
      Err(crossbeam_queue::PopError) => Poll::Pending,
    }
  }
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new(); 