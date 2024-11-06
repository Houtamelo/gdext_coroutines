use std::future::Future;
use godot::obj::WithBaseField;
use godot::prelude::*;
use crate::prelude::*;

pub trait StartAsyncTask {
	/// Starts a new async_task with default settings.
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	/// 
	/// fn showcase_start_async_task(node: Gd<Node3D>) {
	///     node.start_async_task(async {
	///         let result = smol::fs::read_to_string("hello.txt").await;
	///         return result.unwrap();
	///     });
	/// }
	/// ```
	fn start_async_task<R>(
		&self,
		f: impl Future<Output = R> + Send + 'static,
	) -> Gd<SpireCoroutine>
		where
			R: 'static + ToGodot + Send,
	{
		self.async_task(f).spawn()
	}
	
	/// Just like [start_async_task], but does not enforce the `Send` bound.
	/// 
	/// # Safety
	/// 
	/// Caller must ensure that `f` cannot cause data races.
	unsafe fn start_async_task_unchecked<R: 'static + ToGodot>(
		&self,
		f: impl Future<Output = R> + Unpin + 'static
	) -> Gd<SpireCoroutine> {
		self.async_task_unchecked(f).spawn()
	}

	/// Creates a new coroutine builder with default settings.
	///
	/// The coroutine does not actually `spawn` until you call [CoroutineBuilder::spawn].
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use std::time::Duration;
	/// use godot::classes::node::ProcessMode;
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// fn showcase_async_task(node: Gd<Node2D>) {
	///     node.async_task(
	///         async {
	///              smol::Timer::after(Duration::from_secs(5)).await;
	///              return "Profit";
	///         })
	///         .auto_start(false)
	///         .process_mode(ProcessMode::WHEN_PAUSED)
	///         .spawn();
	/// }
	/// ```
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send;
	
	/// Just like [async_task], but does not enforce the `Send` bound.
	/// 
	/// # Safety
	/// 
	/// Caller must ensure that `f` cannot cause data races.
	unsafe fn async_task_unchecked<R: 'static + ToGodot>(
		&self,
		f: impl Future<Output = R> + Unpin + 'static
	) -> CoroutineBuilder<R>;
}

impl<TSelf> StartAsyncTask for Gd<TSelf>
	where
		TSelf: GodotClass + Inherits<Node>,
{
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send,
	{
		CoroutineBuilder::new_async_task(self.clone().upcast(), f)
	}

	unsafe fn async_task_unchecked<R: 'static + ToGodot>(
		&self,
		f: impl Future<Output = R> + Unpin + 'static
	) -> CoroutineBuilder<R> {
		CoroutineBuilder::new_async_task_unchecked(self.clone().upcast(), f)
	}
}

impl<T> StartAsyncTask for &T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send,
	{
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_async_task(base.upcast(), f)
	}

	unsafe fn async_task_unchecked<R: 'static + ToGodot>(
		&self,
		f: impl Future<Output = R> + Unpin + 'static
	) -> CoroutineBuilder<R> {
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_async_task_unchecked(base.upcast(), f)
	}
}

impl<T> StartAsyncTask for &mut T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send,
	{
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_async_task(base.upcast(), f)
	}

	unsafe fn async_task_unchecked<R: 'static + ToGodot>(
		&self,
		f: impl Future<Output = R> + Unpin + 'static
	) -> CoroutineBuilder<R> {
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_async_task_unchecked(base.upcast(), f)
	}
}