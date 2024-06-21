// use std::{fmt, marker::PhantomData, panic::{RefUnwindSafe, UnwindSafe}, rc::Rc};

// use futures::Future;
// use smol::{Executor, Task};

// pub struct LocalExecutor<'a> {
//   /// The inner executor.
//   inner: Executor<'a>,

//   /// Makes the type `!Send` and `!Sync`.
//   _marker: PhantomData<Rc<()>>,
// }

// impl UnwindSafe for LocalExecutor<'_> {}
// impl RefUnwindSafe for LocalExecutor<'_> {}

// impl fmt::Debug for LocalExecutor<'_> {
//   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     write!(f, "LocalExecutor")
//   }
// }

// impl<'a> LocalExecutor<'a> {
//   /// Creates a single-threaded executor.
//   ///
//   /// # Examples
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   ///
//   /// let local_ex = LocalExecutor::new();
//   /// ```
//   pub const fn new() -> LocalExecutor<'a> {
//       LocalExecutor {
//           inner: Executor::new(),
//           _marker: PhantomData,
//       }
//   }

//   /// Returns `true` if there are no unfinished tasks.
//   ///
//   /// # Examples
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   ///
//   /// let local_ex = LocalExecutor::new();
//   /// assert!(local_ex.is_empty());
//   ///
//   /// let task = local_ex.spawn(async {
//   ///     println!("Hello world");
//   /// });
//   /// assert!(!local_ex.is_empty());
//   ///
//   /// assert!(local_ex.try_tick());
//   /// assert!(local_ex.is_empty());
//   /// ```
//   pub fn is_empty(&self) -> bool {
//       self.inner().is_empty()
//   }

//   /// Spawns a task onto the executor.
//   ///
//   /// # Examples
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   ///
//   /// let local_ex = LocalExecutor::new();
//   ///
//   /// let task = local_ex.spawn(async {
//   ///     println!("Hello world");
//   /// });
//   /// ```
//   pub fn spawn<T: 'a>(&self, future: impl Future<Output = T> + 'a) -> Task<T> {
//       let mut active = self.inner().state().active.lock().unwrap();

//       // SAFETY: This executor is not thread safe, so the future and its result
//       //         cannot be sent to another thread.
//       unsafe { self.inner().spawn_inner(future, &mut active) }
//   }

//   /// Spawns many tasks onto the executor.
//   ///
//   /// As opposed to the [`spawn`] method, this locks the executor's inner task lock once and
//   /// spawns all of the tasks in one go. With large amounts of tasks this can improve
//   /// contention.
//   ///
//   /// It is assumed that the iterator provided does not block; blocking iterators can lock up
//   /// the internal mutex and therefore the entire executor. Unlike [`Executor::spawn`], the
//   /// mutex is not released, as there are no other threads that can poll this executor.
//   ///
//   /// ## Example
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   /// use futures_lite::{stream, prelude::*};
//   /// use std::future::ready;
//   ///
//   /// # futures_lite::future::block_on(async {
//   /// let mut ex = LocalExecutor::new();
//   ///
//   /// let futures = [
//   ///     ready(1),
//   ///     ready(2),
//   ///     ready(3)
//   /// ];
//   ///
//   /// // Spawn all of the futures onto the executor at once.
//   /// let mut tasks = vec![];
//   /// ex.spawn_many(futures, &mut tasks);
//   ///
//   /// // Await all of them.
//   /// let results = ex.run(async move {
//   ///     stream::iter(tasks).then(|x| x).collect::<Vec<_>>().await
//   /// }).await;
//   /// assert_eq!(results, [1, 2, 3]);
//   /// # });
//   /// ```
//   ///
//   /// [`spawn`]: LocalExecutor::spawn
//   /// [`Executor::spawn_many`]: Executor::spawn_many
//   pub fn spawn_many<T: Send + 'a, F: Future<Output = T> + Send + 'a>(
//       &self,
//       futures: impl IntoIterator<Item = F>,
//       handles: &mut impl Extend<Task<F::Output>>,
//   ) {
//       let mut active = self.inner().state().active.lock().unwrap();

//       // Convert all of the futures to tasks.
//       let tasks = futures.into_iter().map(|future| {
//           // SAFETY: This executor is not thread safe, so the future and its result
//           //         cannot be sent to another thread.
//           unsafe { self.inner().spawn_inner(future, &mut active) }

//           // As only one thread can spawn or poll tasks at a time, there is no need
//           // to release lock contention here.
//       });

//       // Push them to the user's collection.
//       handles.extend(tasks);
//   }

//   /// Attempts to run a task if at least one is scheduled.
//   ///
//   /// Running a scheduled task means simply polling its future once.
//   ///
//   /// # Examples
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   ///
//   /// let ex = LocalExecutor::new();
//   /// assert!(!ex.try_tick()); // no tasks to run
//   ///
//   /// let task = ex.spawn(async {
//   ///     println!("Hello world");
//   /// });
//   /// assert!(ex.try_tick()); // a task was found
//   /// ```
//   pub fn try_tick(&self) -> bool {
//       self.inner().try_tick()
//   }

//   /// Runs a single task.
//   ///
//   /// Running a task means simply polling its future once.
//   ///
//   /// If no tasks are scheduled when this method is called, it will wait until one is scheduled.
//   ///
//   /// # Examples
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   /// use futures_lite::future;
//   ///
//   /// let ex = LocalExecutor::new();
//   ///
//   /// let task = ex.spawn(async {
//   ///     println!("Hello world");
//   /// });
//   /// future::block_on(ex.tick()); // runs the task
//   /// ```
//   pub async fn tick(&self) {
//       self.inner().tick().await
//   }

//   /// Runs the executor until the given future completes.
//   ///
//   /// # Examples
//   ///
//   /// ```
//   /// use async_executor::LocalExecutor;
//   /// use futures_lite::future;
//   ///
//   /// let local_ex = LocalExecutor::new();
//   ///
//   /// let task = local_ex.spawn(async { 1 + 2 });
//   /// let res = future::block_on(local_ex.run(async { task.await * 2 }));
//   ///
//   /// assert_eq!(res, 6);
//   /// ```
//   pub async fn run<T>(&self, future: impl Future<Output = T>) -> T {
//       self.inner().run(future).await
//   }

//   /// Returns a reference to the inner executor.
//   pub fn inner(&self) -> &Executor<'a> {
//       &self.inner
//   }
// }

// impl<'a> Default for LocalExecutor<'a> {
//   fn default() -> LocalExecutor<'a> {
//       LocalExecutor::new()
//   }
// }
