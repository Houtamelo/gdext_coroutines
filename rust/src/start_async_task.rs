use std::future::Future;
use godot::meta::conv::ObjectToOwned;
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
    ///     let sig_future = node.signals().tree_entered().to_future(); 
	///     node.start_async_task({
    ///         async {
    ///             sig_future.await;
    ///             godot_print!("Entered tree!");
    ///         }
    ///     });
	/// }
	/// ```
	fn start_async_task<R>(
		&self,
		f: impl Future<Output = R> + 'static,
	) -> Gd<SpireCoroutine>
		where
			R: 'static + ToGodot,
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
    /// use godot::prelude::*;
    /// use gdext_coroutines::prelude::*;
	///
    /// fn showcase_start_async_task(node: Gd<Node3D>) {
    ///     let sig_future = node.signals().tree_entered().to_future(); 
    ///     node.start_async_task({
    ///         async {
    ///             sig_future.await;
    ///             godot_print!("Entered tree!");
    ///         }
    ///     });
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