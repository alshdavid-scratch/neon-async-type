use std::time::Duration;

use futures::Future;
use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::handle::Handle;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::result::Throw;
use neon::types::JsBoolean;
use neon::types::JsFunction;
use neon::types::JsUndefined;
use neon::types::JsValue;
use neon::types::Value;
use once_cell::unsync::Lazy;

thread_local! {
  static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap());
  static LOCAL_SET: Lazy<tokio::task::LocalSet> = Lazy::new(|| tokio::task::LocalSet::new());
}

pub fn spawn_async<F, R>(
  future: F,
) -> R where
  F: Future<Output = R>, {
  LOCAL_SET.with(|ls| RUNTIME.with(|rt| rt.block_on(async move { ls.run_until(future).await })))
}

async fn foo<'a>(mut cx: FunctionContext<'a>) -> JsResult<'a, JsBoolean> {
  Ok(cx.boolean(false))  
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  export_function_async(&mut cx, "foo", foo)?;

  neon::registered().export(&mut cx)?;
  Ok(())
}

fn export_function_async<'a, Func, Fut, Ret>(
  cx: &mut ModuleContext,
  name: &str,
  f: Func,
) -> NeonResult<()>
where
  Func: Copy + Fn(FunctionContext<'a>) -> Fut + 'static,
  Fut: Future<Output = Result<Handle<'a, Ret>, Throw>>,
  Ret: Value,
{
  let value = JsFunction::new(cx,  move |cx: FunctionContext| {
    spawn_async(async move {
      // Ok(cx.undefined()) // Works
      foo(cx).await // Works
      // f(cx).await // Does not work
    })
  })?
  .upcast::<JsValue>();

  cx.export_value(name, value)?;
  Ok(())
}
