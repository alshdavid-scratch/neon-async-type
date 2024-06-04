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
  static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap());
  static LOCAL_SET: Lazy<tokio::task::LocalSet> = Lazy::new(|| tokio::task::LocalSet::new());
}

// This still blocks the main thread
pub fn spawn_async<F, R>(future: F) -> R
where
  F: Future<Output = R>,
{
  LOCAL_SET.with(|ls| RUNTIME.with(|rt| rt.block_on(async move { ls.run_until(future).await })))
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
  let value = JsFunction::new(cx, move |mut cx: FunctionContext| {
    let target = JsFunction::new(&mut cx, move |cx| spawn_async(async move { f(cx).await }))?;

    let mut callee = cx.global::<JsFunction>("setTimeout")?.call_with(&mut cx);

    callee.arg(target);
    callee.arg(cx.number(0));

    let mut i = 0;
    loop {
      let Some(arg) = cx.argument_opt(i) else { break };
      callee.arg(arg);
      i += 1;
    }

    callee.exec(&mut cx)?;

    Ok(cx.undefined())
  })?
  .upcast::<JsValue>();

  cx.export_value(name, value)?;
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
