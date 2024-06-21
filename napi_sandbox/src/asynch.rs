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
use smol::LocalExecutor;

thread_local! {
  static EXECUTOR: Lazy<LocalExecutor<'static>> = Lazy::new(|| smol::LocalExecutor::new());
}

pub fn spawn_async_local<F>(future: F)
where
  F: Future<Output = ()> + 'static,
{
  EXECUTOR.with(|ex| {
    ex.spawn(async move {
      future.await;
      return ();
    })
    .detach();
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


    // spawn_async_local(async move {
    //   f(cx).await;
    // });
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
