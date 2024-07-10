use std::future::Future;
use std::ops::Coroutine;
use godot::prelude::*;

use crate::prelude::*;

fn async_to_coroutine<T: ToGodot<Via = TVia> + Send + 'static, TVia>(
	f: impl Future<Output = T> + Send + 'static,
) -> impl Coroutine<(), Yield = Yield, Return = Variant> {
	let task = smol::spawn(f);

	#[coroutine] move || {
		while !task.is_finished() {
			yield frames(1);
		}

		smol::block_on(task).to_variant()
	}
}

impl<T: ToGodot> CoroutineBuilder<T> {
	/// Works similarly to [spawn], except it accepts an [async function](Future) as input.
	///
	/// Async functions are run in background threads using the crate [smol].
	/// 
	/// # Example
	/// 
	/// ```
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// let node = Node::new_alloc();
	/// 
	/// node.coroutine()
	///     .spawn_async_fn(
	///         async {
	///             smol::Timer::after(std::time::Duration::from_secs(5)).await;
	///             godot_print!("Profit :3"); 
	///         });
	/// ```
	pub fn spawn_async_fn(
		self,
		f: impl Future<Output = T> + Send + 'static,
	) -> Gd<GodotCoroutine>
		where
			T: Send + 'static
	{
		let f = async_to_coroutine(f);

		let mut coroutine =
			Gd::from_init_fn(|base| {
				GodotCoroutine {
					base,
					coroutine: Box::pin(f),
					poll_mode: self.poll_mode,
					last_yield: None,
					paused: !self.auto_start,
				}
			});

		coroutine.set_process_priority(256);
		coroutine.set_physics_process_priority(256);

		coroutine.set_process_mode(self.process_mode);

		let mut owner = self.owner;
		owner.add_child(coroutine.clone().upcast());

		coroutine
	}
}