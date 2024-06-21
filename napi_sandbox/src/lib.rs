#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate libuv_sys2 as uv;

mod ok;
mod asynch;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use futures::Future;
use neon::context::{Context, FunctionContext, ModuleContext};
use neon::handle::Handle;
use neon::result::{JsResult, NeonResult};
use neon::sys::bindings::{get_uv_event_loop, Callback};
// use neon::sys::bindings::UvEventLoop;
// use ok::new_executor_and_spawner;
// use once_cell::sync::Lazy as LazySync;
// use once_cell::unsync::Lazy;
// use smol::{io, net, prelude::*, Unblock};
// use async_compat::CompatExt;
use async_io::{Async, Timer};
use neon::types::{JsFunction, JsNumber, JsObject, JsPromise, JsUndefined};
use neon::object::Object;
use smol::LocalExecutor;
use uv::{uv_idle_t, uv_loop_t};

fn start(mut cx: FunctionContext) -> JsResult<JsUndefined> {
  let cx_ptr = cx.to_raw();
  let mut result = MaybeUninit::uninit();
  unsafe { get_uv_event_loop(cx_ptr.clone(), result.as_mut_ptr()) };
  let ptr = unsafe { *result.as_mut_ptr() };
  let ptr = ptr as *mut uv_loop_t;
  let h = unsafe { libuv::r#loop::Loop::from_external(ptr) };
  let mut idle_handle = h.idle().unwrap();
  let ex = smol::LocalExecutor::new();

  let callback = cx.argument::<JsFunction>(0)?;
  // callback.call_with(&cx).exec(&mut cx)?;

  spawn_async_local(&ex, Box::pin(async move {
    callback.call_with(&cx).exec(&mut cx);
    println!("R 1");
    Timer::after(Duration::from_secs(1)).await;
    println!("R 2");
    ()
  }));

  idle_handle.start(move |mut idle_handle: libuv::IdleHandle| {
    if ex.is_empty() {
      idle_handle.stop().unwrap();
      return;
    }
    ex.try_tick();
  }).unwrap();


  Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("start", start)?;
  Ok(())
}

pub fn spawn_async_local<F>(ex: &LocalExecutor, future: Pin<Box<F>>)
where
  F: Future<Output = ()> + 'static,
{
  ex.spawn(future).detach();
}

// fn console_log(cx: Rc<RefCell<FunctionContext>>) -> NeonResult<()> {
//   let console: Handle<JsObject> = cx.global("console")?;
//   let log: Handle<JsFunction> = console.get(&mut *cx, "log")?;

//   let v = cx.string("From Rust");
//   log.call_with(&mut *cx).arg(v).exec(&mut *cx)?;
//   Ok(())
// }

// fn executor(cx: &mut FunctionContext) -> LocalExecutor<'static> {
//   let mut result = MaybeUninit::uninit();
//   unsafe { get_uv_event_loop(cx.to_raw(), result.as_mut_ptr()) };
//   let ptr = unsafe { *result.as_mut_ptr() };
//   let ptr = ptr as *mut uv_loop_t;

//   let mut h = libuv::r#loop::Loop::connect(ptr);
//   let mut idle_handle = h.idle().unwrap();

//   smol::LocalExecutor::new()
// }