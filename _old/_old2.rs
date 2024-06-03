use std::time::Duration;

use futures::Future;
use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::types::JsFunction;
use neon::types::JsUndefined;
use once_cell::unsync::Lazy;

thread_local! {
  static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap());
  static LOCAL_SET: Lazy<tokio::task::LocalSet> = Lazy::new(|| tokio::task::LocalSet::new());
}

pub fn spawn_async<'a, F, C>(cx: &'a mut C, future: F)
where
  F: Future + 'static,
  C: Context<'a>
{
  let timer = cx.global::<JsFunction>("setTimeout");
  LOCAL_SET.with(|ls| RUNTIME.with(|rt| rt.block_on(async move { ls.run_until(future).await })));
}

fn foo(mut cx: FunctionContext) -> JsResult<JsUndefined> {
  spawn_async(&mut cx, async move {
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Hello");
  });

  Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("foo", foo)?;
  neon::registered().export(&mut cx)?;
  Ok(())
}


fn export_function_async<F>(cx: &mut ModuleContext, name: &str, f: F) {
  
}