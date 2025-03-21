// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

//! This module helps deno implement timers and performance APIs.

use deno_core::error::AnyError;
use deno_core::op;
use deno_core::ZeroCopyBuf;

use deno_core::CancelFuture;
use deno_core::CancelHandle;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

pub trait TimersPermission {
  fn allow_hrtime(&mut self) -> bool;
  fn check_unstable(&self, state: &OpState, api_name: &'static str);
}

pub type StartTime = Instant;

static mut NOW_BUF: *mut u32 = std::ptr::null_mut();

#[op]
pub fn op_now_set_buf(buf: ZeroCopyBuf) {
  assert_eq!(buf.len(), 8);
  // SAFETY: This is safe because this is the only place where we initialize
  // NOW_BUF.
  unsafe {
    NOW_BUF = buf.as_ptr() as *mut u32;
  }
}

// Returns a milliseconds and nanoseconds subsec
// since the start time of the deno runtime.
// If the High precision flag is not set, the
// nanoseconds are rounded on 2ms.
#[op(fast)]
pub fn op_now<TP>(state: &mut OpState)
where
  TP: TimersPermission + 'static,
{
  let start_time = state.borrow::<StartTime>();
  let elapsed = start_time.elapsed();
  let seconds = elapsed.as_secs();
  let mut subsec_nanos = elapsed.subsec_nanos();

  // If the permission is not enabled
  // Round the nano result on 2 milliseconds
  // see: https://developer.mozilla.org/en-US/docs/Web/API/DOMHighResTimeStamp#Reduced_time_precision
  if !state.borrow_mut::<TP>().allow_hrtime() {
    let reduced_time_precision = 2_000_000; // 2ms in nanoseconds
    subsec_nanos -= subsec_nanos % reduced_time_precision;
  }

  // SAFETY: This is safe because we initialize NOW_BUF in op_now_set_buf, its a null pointer
  // otherwise.
  // op_now_set_buf guarantees that the buffer is 8 bytes long.
  unsafe {
    if !NOW_BUF.is_null() {
      let buf = std::slice::from_raw_parts_mut(NOW_BUF, 2);
      buf[0] = seconds as u32;
      buf[1] = subsec_nanos as u32;
    }
  }
}

pub struct TimerHandle(Rc<CancelHandle>);

impl Resource for TimerHandle {
  fn name(&self) -> Cow<str> {
    "timer".into()
  }

  fn close(self: Rc<Self>) {
    self.0.cancel();
  }
}

/// Creates a [`TimerHandle`] resource that can be used to cancel invocations of
/// [`op_sleep`].
#[op]
pub fn op_timer_handle(state: &mut OpState) -> ResourceId {
  state
    .resource_table
    .add(TimerHandle(CancelHandle::new_rc()))
}

/// Waits asynchronously until either `millis` milliseconds have passed or the
/// [`TimerHandle`] resource given by `rid` has been canceled.
#[op]
pub async fn op_sleep(
  state: Rc<RefCell<OpState>>,
  millis: u64,
  rid: ResourceId,
) -> Result<(), AnyError> {
  let handle = state.borrow().resource_table.get::<TimerHandle>(rid)?;
  tokio::time::sleep(Duration::from_millis(millis))
    .or_cancel(handle.0.clone())
    .await?;
  Ok(())
}
