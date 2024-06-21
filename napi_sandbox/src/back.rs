  let mut result = MaybeUninit::uninit();
  unsafe { get_uv_event_loop(cx.to_raw(), result.as_mut_ptr()) };
  let ptr = unsafe { *result.as_mut_ptr() };
  let ptr = ptr as *mut uv_loop_t;

  let mut h = libuv::r#loop::Loop::connect(ptr);
  let mut idle_handle = h.idle().unwrap();

  let ex = smol::LocalExecutor::new();

  let cx = Rc::new(RefCell::new(cx));

  ex.spawn(async {
    println!("R 1");
    Timer::after(Duration::from_secs(1)).await;
    println!("R 2");
    console_log(cx);
  }).detach();

  idle_handle.start(move |mut idle_handle: libuv::IdleHandle| {
    if ex.is_empty() {
      idle_handle.stop().unwrap();
      return;
    }
    ex.try_tick();
  }).unwrap();
