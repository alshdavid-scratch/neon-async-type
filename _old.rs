use std::cell::RefCell;
use std::rc::Rc;

use futures::executor::LocalPool;
use futures::executor::LocalSpawner;
use futures::task::LocalSpawnExt;
use futures::task::SpawnError;
use futures::Future;
use futures::;
use neon::context::Context;
use neon::context::FunctionContext;
use neon::context::ModuleContext;
use neon::result::JsResult;
use neon::result::NeonResult;
use neon::types::JsPromise;
use once_cell::unsync::Lazy;

thread_local! {
  static RUNTIME: Lazy<RefCell<LocalPool>> = Lazy::new(|| RefCell::new(LocalPool::new()));
  static SPAWNER: Lazy<LocalSpawner> = Lazy::new(|| RUNTIME.with(|r| r.borrow().spawner()));
}

fn spawn_async<Fut>(future: Fut) -> Result<(), SpawnError>
where
  Fut: Future<Output = ()> + 'static,
{
  let result = SPAWNER.with(|s| s.spawn_local(future));
  RUNTIME.with(|r| r.borrow_mut().run_until_stalled());
  result
}

fn foo(mut cx: FunctionContext) -> JsResult<JsPromise> {
  let cx_ref = Rc::new(RefCell::new(cx));
  let mut cx = cx_ref.borrow_mut();

  let name = cx.string("Dynamically Generated");

  let (deferred, promise) = cx.promise();

  spawn_async(async move {
    println!("Hello");
    deferred.resolve(&mut cx, name)
  }).unwrap();

  Ok(promise)
  // Ok(name)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("foo", foo)?;
  neon::registered().export(&mut cx)?;
  Ok(())
}
