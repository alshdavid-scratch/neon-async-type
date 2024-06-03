use std::time::Duration;

use futures::Future;
use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::types::JsFunction;
use neon::types::JsUndefined;
use neon::types::JsValue;
use once_cell::unsync::Lazy;

thread_local! {
  static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap());
  static LOCAL_SET: Lazy<tokio::task::LocalSet> = Lazy::new(|| tokio::task::LocalSet::new());
}

pub fn spawn_async<F>(
  future: F,
) where
  F: Future + 'static,
{
  LOCAL_SET.with(|ls| RUNTIME.with(|rt| rt.block_on(async move { ls.run_until(future).await })));
}

async fn foo() -> bool {
  println!("hi");
  true
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  export_function_async(&mut cx, "foo", foo)?;

  neon::registered().export(&mut cx)?;
  Ok(())
}

fn export_function_async<F, Fut>(
  cx: &mut ModuleContext,
  name: &str,
  f: F,
) -> NeonResult<()>
where
  F: Copy + Fn() -> Fut + 'static,
  Fut: Future<Output = bool>
{
  let value = JsFunction::new(cx,  move |mut cx| {
    let target = JsFunction::new(&mut cx, move |mut cx| {
      spawn_async(async move {
        f().await;
      });
      Ok(cx.undefined())
    })?;

    // Run non blocking
    cx.global::<JsFunction>("setTimeout")?
      .call_with(&mut cx)
      .arg(target)
      .exec(&mut cx)?;

    Ok(cx.undefined())
  })?
  .upcast::<JsValue>();

  cx.export_value(name, value)?;
  Ok(())
}
