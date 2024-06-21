mod asynch;
mod local_ex;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::time::Duration;

use async_io::Timer;
use futures::Future;
use libuv::sys::uv_idle_t;
use libuv::sys::uv_loop_t;
use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::context::SysContext;
use neon::handle::Handle;
use neon::object::Object;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::sys::bindings::get_uv_event_loop;
use neon::types::JsFunction;
use neon::types::JsNumber;
use neon::types::JsObject;
use neon::types::JsPromise;
use neon::types::JsUndefined;
use once_cell::sync::Lazy as LazySync;
use once_cell::unsync::Lazy;
use smol::LocalExecutor;
use tokio::task::LocalSet;

thread_local! {
  static EXECUTOR: Lazy<LocalExecutor<'static>> = Lazy::new(|| smol::LocalExecutor::new());
}

static RUNTIME: LazySync<tokio::runtime::Runtime> = LazySync::new(|| {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
});

thread_local! {
  static LOCAL_SET: Lazy<tokio::task::LocalSet> = Lazy::new(|| tokio::task::LocalSet::new());
}

fn start(mut cx: FunctionContext) -> JsResult<JsUndefined> {
  let uv = get_lib_uv(&cx);

  let callback = cx.argument::<JsFunction>(0)?;
  cx.global_object().set(&mut cx, "callback", callback)?;

  // cx.compute_scoped(|mut cx| {
  //   let global = cx.global_object();
  //   Ok(cx.undefined())
  // })?;

  cx.execute_async(move |mut ecx| Box::pin(async move {
    let global = ecx.global_object();
    let callback: Handle<JsFunction> = global.get(&mut ecx, "callback").unwrap();

    callback.call_with(&mut ecx).exec(&mut ecx).unwrap();
    println!("R 1.1");
    Timer::after(Duration::from_secs(1)).await;
    println!("R 2.1");
  }));

  // cx.execute_async(move |mut cx| Box::pin(async move {
  //   // callback;
  //   println!("R 1.2");
  //   Timer::after(Duration::from_secs(1)).await;
  //   println!("R 2.2");
  // }));

  Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("start", start)?;
  Ok(())
}

pub fn get_lib_uv<'a>(cx: &impl Context<'a>) -> libuv::r#loop::Loop {
  let mut result = MaybeUninit::uninit();
  unsafe { get_uv_event_loop(cx.to_raw(), result.as_mut_ptr()) };
  let ptr = unsafe { *result.as_mut_ptr() };
  let ptr = ptr as *mut uv_loop_t;
  unsafe { libuv::r#loop::Loop::from_external(ptr) }
}

pub fn spawn_async_local<F>(
  ex: &LocalExecutor,
  future: Pin<Box<F>>,
) where
  F: Future<Output = ()> + 'static,
{
  ex.spawn(future).detach();
}
