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
		f: impl std::future::Future<Output = R> + Send + 'static,
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
		f: impl std::future::Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send;
}

impl<TSelf> StartAsyncTask for Gd<TSelf>
	where
		TSelf: GodotClass + Inherits<Node>,
{
	fn async_task<R>(
		&self,
		f: impl std::future::Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send,
	{
		CoroutineBuilder::new_async_task(self.clone().upcast(), f)
	}
}

impl<'a, T> StartAsyncTask for &'a T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn async_task<R>(
		&self,
		f: impl std::future::Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send,
	{
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_async_task(base.upcast(), f)
	}
}

impl<'a, T> StartAsyncTask for &'a mut T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn async_task<R>(
		&self,
		f: impl std::future::Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot + Send,
	{
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_async_task(base.upcast(), f)
	}
}