use std::ops::Coroutine;
use godot::obj::WithBaseField;
use godot::prelude::*;
use crate::prelude::*;
use crate::yielding::SpireYield;

pub trait StartCoroutine {
	/// Spawns and starts a new coroutine with default settings.
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// fn showcase_start_coroutine(node: Gd<Node2D>) {
	///     node.start_coroutine(
	///         #[coroutine] || {
	///             yield frames(5);
	///             godot_print!("5 frames passed!");
	///         });
	/// }
	/// ```
	/// 
	/// # On Panics
	/// If `f` panics, the SpireCoroutine will automatically self-destruct and the closure will be leaked
	fn start_coroutine<R>(
		&self,
		f: impl 'static + Unpin + Coroutine<(), Yield = SpireYield, Return = R>,
	) -> Gd<SpireCoroutine>
		where
			R: 'static + ToGodot,
	{
		self.coroutine(f).spawn()
	}

	/// Creates a new coroutine builder with default settings.
	///
	/// The coroutine does not actually `spawn` until you call [CoroutineBuilder::spawn].
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::classes::node::ProcessMode;
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// fn showcase_coroutine(node: Gd<Node2D>) {
	///     node.coroutine(
	///         #[coroutine] || {
	///             godot_print!("This is a customized coroutine!");
	///             yield seconds(2.0);
	///             godot_print!("2 seconds passed!");
	///         })
	///         .auto_start(false)
	///         .process_mode(ProcessMode::WHEN_PAUSED)
	///         .spawn();
	/// }
	/// ```
	/// 
	/// # On Panics
	/// If `f` panics, the SpireCoroutine will automatically self-destruct and the closure will be leaked
	fn coroutine<R>(
		&self,
		f: impl 'static + Unpin + Coroutine<(), Yield = SpireYield, Return = R>,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot;
}

impl<TSelf> StartCoroutine for Gd<TSelf>
	where
		TSelf: GodotClass + Inherits<Node>,
{
	fn coroutine<R>(
		&self,
		f: impl 'static + Unpin + Coroutine<(), Yield = SpireYield, Return = R>,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot,
	{
		CoroutineBuilder::new_coroutine(self.clone().upcast(), f)
	}
}

impl<T> StartCoroutine for &T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn coroutine<R>(
		&self,
		f: impl 'static + Unpin + Coroutine<(), Yield = SpireYield, Return = R>,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot,
	{
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_coroutine(base.upcast(), f)
	}
}

impl<T> StartCoroutine for &mut T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn coroutine<R>(
		&self,
		f: impl 'static + Unpin + Coroutine<(), Yield = SpireYield, Return = R>,
	) -> CoroutineBuilder<R>
		where
			R: 'static + ToGodot,
	{
		let base = self.base_field().to_gd();
		CoroutineBuilder::new_coroutine(base.upcast(), f)
	}
}