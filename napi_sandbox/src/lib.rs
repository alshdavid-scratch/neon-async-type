mod asynch;

use std::time::Duration;

use asynch::export_function_async;
use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::handle::Handle;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::types::JsFunction;
use neon::types::JsString;
use neon::types::JsUndefined;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  export_function_async(&mut cx, "countAsync", count_async)?;
  neon::registered().export(&mut cx)?;
  Ok(())
}

async fn count_async<'a>(mut cx: FunctionContext<'a>) -> JsResult<'a, JsUndefined> {
  let name: Handle<JsString> = cx.argument(0)?;
  let callback: Handle<JsFunction> = cx.argument(1)?;

  for i in 0..4 {
    tokio::time::sleep(Duration::from_millis(200)).await;
    callback.call_with(&mut cx)
      .arg(name)
      .arg(cx.number(i))
      .exec(&mut cx)?;
  }

  Ok(cx.undefined())
}
