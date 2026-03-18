use std::future::Future;
use godot::meta::ObjectToOwned;
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
		f: impl Future<Output = R> + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot;
}

impl<TSelf> StartAsyncTask for Gd<TSelf>
	where
		TSelf: GodotClass + Inherits<Node>,
{
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot,
	{
		CoroutineBuilder::new_async_task(self.clone().upcast(), f)
	}
}

impl<T> StartAsyncTask for &T
	where
		T: WithBaseField + Inherits<Node>,
{
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot,
	{
		let base = self.object_to_owned();
		CoroutineBuilder::new_async_task(base.upcast(), f)
	}
}

impl<T> StartAsyncTask for &mut T
	where
		T: WithBaseField + Inherits<Node>,
{
	fn async_task<R>(
		&self,
		f: impl Future<Output = R> + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot,
	{
		let base = self.object_to_owned();
		CoroutineBuilder::new_async_task(base.upcast(), f)
	}
}