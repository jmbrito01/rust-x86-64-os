use alloc::{string::String, vec::Vec};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::stream::StreamExt;

use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;

use crate::{kernel::{command_line}, kprintln};  

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn push_command(command: &str, args: Vec<String>) {
  if let Ok(queue) = SCANCODE_QUEUE.try_get() {
    if let Err(_) = queue.push((String::from(command), args)) {
      kprintln!("WARNING: scancode queue full; dropping keyboard input");
    } else {
      WAKER.wake();
    }
  } else {
    kprintln!("WARNING: scancode queue uninitialized");
  }
}

pub async fn handle_command_runs() {
  let mut commands_to_run = CommandLineStream::new();
  while let Some(command_to_run) = commands_to_run.next().await {
    command_line::run_command(command_to_run.0.as_str(), command_to_run.1).await
  }
}

pub struct CommandLineStream {
  _private: (),
}

impl CommandLineStream {
  pub fn new() -> Self {
    SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
      .expect("CommandLineStream::new should only be called once");
    CommandLineStream { _private: () }
  }
}

impl Stream for CommandLineStream {
  type Item = (String, Vec<String>);

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<(String, Vec<String>)>> {
    let queue = SCANCODE_QUEUE
      .try_get()
      .expect("command line queue not initialized");
    // fast path
    if let Ok(queued_command) = queue.pop() {
        return Poll::Ready(Some(queued_command));
    }

    WAKER.register(&cx.waker());
    match queue.pop() {
      Ok(queued_command) => {
        WAKER.take();
        Poll::Ready(Some(queued_command))
      }
      Err(crossbeam_queue::PopError) => Poll::Pending,
    }
  }
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<(String, Vec<String>)>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new(); 