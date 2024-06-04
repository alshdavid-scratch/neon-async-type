use std::future::Future;

use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::handle::Handle;
use neon::result::NeonResult;
use neon::result::Throw;
use neon::types::JsFunction;
use neon::types::JsValue;
use neon::types::Value;
use once_cell::unsync::Lazy;

thread_local! {
  static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap());
  static LOCAL_SET: Lazy<tokio::task::LocalSet> = Lazy::new(|| tokio::task::LocalSet::new());
}

pub fn spawn_async_local<F, R>(future: F) -> R
where
  F: Future<Output = R>,
{
  // tokio runtime to execute the futures
  RUNTIME.with(|rt| {
    // LocalSet to spawn !Sync futures on the main thread
    LOCAL_SET.with(|ls| {
      // Execute the futures
      //    Note: this still blocks the main thread until the futures
      //          are complete. I'm hoping to find a way around this
      rt.block_on(async move {
        // Run the target async code
        ls.run_until(future).await
      })
    })
  })
}

pub fn export_function_async<'a, Func, Ret>(
  cx: &mut ModuleContext,
  name: &str,
  f: Func,
) -> NeonResult<()>
where
  Func: for<'any> AsyncNeonFunction<'any, Ret> + Copy + 'static,
  Ret: Value,
{
  /*
    This essentially creates:
    export const function_name = (...args) => setTimeout(wrapped_native_async_fn, 0, ...args)
  */

  // Create a wrapper function for the incoming async function
  let wrapper = JsFunction::new(cx, move |mut cx: FunctionContext| {
    // Create a wrapper for the async execution
    let target = JsFunction::new(&mut cx, move |cx| spawn_async_local(async move { f(cx).await }))?;

    // Run the target function within a globalThis.setTimeout(target, 0, ...args)
    let mut set_timeout = cx.global::<JsFunction>("setTimeout")?.call_with(&mut cx);

    set_timeout.arg(target);
    set_timeout.arg(cx.number(0));

    let mut i = 0;
    loop {
      let Some(arg) = cx.argument_opt(i) else { break };
      set_timeout.arg(arg);
      i += 1;
    }

    set_timeout.exec(&mut cx)?;

    Ok(cx.undefined())
  })?
  .upcast::<JsValue>();

  cx.export_value(name, wrapper)?;
  Ok(())
}

pub trait AsyncNeonFunction<'a, R: Value>: Fn(FunctionContext<'a>) -> Self::Fut {
  type Fut: Future<Output = Result<Handle<'a, R>, Throw>>;
}

impl<'a, F, Fut, R: Value> AsyncNeonFunction<'a, R> for F
where
  F: Fn(FunctionContext<'a>) -> Fut,
  Fut: Future<Output = Result<Handle<'a, R>, Throw>>,
{
  type Fut = Fut;
}
