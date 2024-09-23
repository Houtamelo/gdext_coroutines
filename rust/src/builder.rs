use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

use godot::classes::node::ProcessMode;
use godot::prelude::*;

use crate::OnFinishCall;
use crate::prelude::*;
use crate::yielding::SpireYield;

/// Builder struct for customizing coroutine behavior.
#[must_use]
pub struct CoroutineBuilder<R: 'static + ToGodot = ()> {
	pub(crate) f: Box<dyn Unpin + Coroutine<(), Yield = SpireYield, Return = Variant>>,
	pub(crate) owner: Gd<Node>,
	/// Determines if the coroutine should be polled in [_process](INode::process)
	/// or [_physics_process](INode::physics_process)
	pub(crate) poll_mode: PollMode,
	/// Godot [ProcessMode] which the coroutine should run in.
	pub(crate) process_mode: ProcessMode,
	/// Whether the coroutine should be started automatically.
	pub(crate) auto_start: bool,
	/// A list of callables to invoke when the coroutine finishes.
	///
	/// The callables will be invoked with the coroutine's return value as a Variant.
	pub(crate) calls_on_finish: Vec<OnFinishCall>,
	/// Type hint for the coroutine's return value.
	pub(crate) type_hint: std::marker::PhantomData<R>,
}

impl<R> CoroutineBuilder<R>
	where
		R: 'static + ToGodot,
{
	/// Creates a new coroutine builder with default settings.
	#[doc(hidden)]
	pub fn new_coroutine(
		owner: Gd<Node>,
		f: impl 'static + Unpin + Coroutine<(), Yield = SpireYield, Return = R>,
	) -> CoroutineBuilder<R> {
		let wrapper =
			#[coroutine] move || {
				let mut f = f;

				loop {
					let pin = Pin::new(&mut f);
					match pin.resume(()) {
						CoroutineState::Yielded(_yield) => {
							yield _yield;
						}
						CoroutineState::Complete(result) => {
							return result.to_variant();
						}
					}
				}
			};

		Self {
			f: Box::new(wrapper),
			owner,
			poll_mode: PollMode::Process,
			process_mode: ProcessMode::INHERIT,
			auto_start: true,
			calls_on_finish: Vec::new(),
			type_hint: std::marker::PhantomData,
		}
	}
	
	/// Creates a new coroutine builder with default settings.
	/// 
	/// Instead of running a regular Rust Coroutine, this runs a [Future](std::future::Future) in a background thread.
	#[cfg(feature = "async")]
	#[doc(hidden)]
	pub fn new_async_task(
		owner: Gd<Node>,
		f: impl std::future::Future<Output = R> + Send + 'static,
	) -> CoroutineBuilder<R>
		where
			R: Send,
	{
		let task = smol::spawn(f);

		let routine =
			#[coroutine] move || {
				while !task.is_finished() {
					yield frames(1);
				}

				smol::block_on(task).to_variant()
			};

		CoroutineBuilder {
			f: Box::new(routine),
			owner,
			poll_mode: PollMode::Process,
			process_mode: ProcessMode::INHERIT,
			auto_start: true,
			calls_on_finish: Vec::new(),
			type_hint: std::marker::PhantomData,
		}
	}

	/// Whether the coroutine should be started automatically upon spawning.
	///
	/// If false, you'll have to manually call [SpireCoroutine::resume] after spawning.
	pub fn auto_start(self, auto_start: bool) -> Self {
		Self {
			auto_start,
			..self
		}
	}

	/// Godot [ProcessMode] which the coroutine should run in.
	pub fn process_mode(self, process_mode: ProcessMode) -> Self {
		Self {
			process_mode,
			..self
		}
	}

	/// Determines if the coroutine should be polled in [_process](INode::process)
	/// or [_physics_process](INode::physics_process)
	pub fn poll_mode(self, poll_mode: PollMode) -> Self {
		Self {
			poll_mode,
			..self
		}
	}

	/// Adds `f` to the list of closures that will be invoked when the coroutine finishes.
	///
	/// The return value of the coroutine(`T`) will be passed to `f`.
	/// 
	/// [finished](SIGNAL_FINISHED) is only emitted if the coroutine finishes normally.
	/// 
	/// Some cases where the coroutine's end is considered "abnormal":
	/// - The parent node of the coroutine was deleted (freed)
	/// - The coroutine's main closure panics
	/// - The coroutine ends with [force_run_to_completion](SpireCoroutine::force_run_to_completion)
	/// - The coroutine ends with [kill](SpireCoroutine::kill)
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// fn showcase_finish(node: Gd<Node2D>) {
	///     node.coroutine(
	///         #[coroutine] || { 
	///             yield frames(2);
	///             return 5;
	///         })
	///         .on_finish(|res| println!("Coroutine finished, result: {res}"))
	///         .spawn();
	/// }
	/// ```
	pub fn on_finish(
		self,
		f: impl 'static + FnOnce(R),
	) -> Self
		where
			R: FromGodot,
	{
		let wrapper =
			move |var: Variant| {
				match var.try_to::<R>() {
					Ok(r) => { f(r); }
					Err(err) => {
						godot_error!("{err}");
					}
				}
			};

		let mut calls_on_finish = self.calls_on_finish;
		calls_on_finish.push(OnFinishCall::Closure(Box::new(wrapper)));

		Self {
			calls_on_finish,
			..self
		}
	}

	/// See [on_finish](SpireCoroutine::on_finish)
	/// 
	/// This variant takes a [Callable] instead of a closure.
	pub fn on_finish_callable(
		self,
		callable: Callable,
	) -> Self
		where
			R: FromGodot,
	{
		let mut calls_on_finish = self.calls_on_finish;
		calls_on_finish.push(OnFinishCall::Callable(callable));

		Self {
			calls_on_finish,
			..self
		}
	}

	/// Completes the builder, spawning the coroutine's executor.
	///
	/// The executor is the type [SpireCoroutine], a node that will be added as a child of `owner`.
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// fn showcase_spawn(node: Gd<Node2D>) {
	///     let builder = 
	///         node.coroutine(
	///             #[coroutine] || {
	///                 yield frames(5);
	///                 yield seconds(5.0);
	///             });
	///
	///     let coroutine: Gd<SpireCoroutine> = builder.spawn();
	/// }
	/// ```
	pub fn spawn(self) -> Gd<SpireCoroutine> {
		let mut coroutine =
			Gd::from_init_fn(|base| {
				SpireCoroutine {
					base,
					coroutine: self.f,
					poll_mode: self.poll_mode,
					last_yield: None,
					paused: !self.auto_start,
					calls_on_finish: self.calls_on_finish,
				}
			});

		coroutine.set_process_priority(256);
		coroutine.set_physics_process_priority(256);

		coroutine.set_process_mode(self.process_mode);

		let mut owner = self.owner;
		owner.add_child(coroutine.clone());

		coroutine
	}
}