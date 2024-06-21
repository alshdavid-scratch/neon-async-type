mod asynch;
mod local_ex;

use std::time::Duration;

use neon::context::AsyncFunctionContext;
use neon::context::Context;
use neon::context::ModuleContext;
use neon::handle::Handle;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::types::JsFunction;
use neon::types::JsString;
use smol::Timer;

async fn foo<'a>(mut cx: AsyncFunctionContext) -> JsResult<'a, JsString> {
  let callback: Handle<JsFunction> = cx.argument(0)?;
  
  Timer::after(Duration::from_secs(1)).await;
  callback.call_with(&cx).exec(&mut cx)?;
  
  Ok(cx.string("promise resolved"))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function_async("foo", foo)?;
  Ok(())
}
