use std::marker::PhantomData;
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;
use godot::classes::node::ProcessMode;
use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::prelude::*;

/// A Godot class responsible for managing a coroutine.
///
/// This should not be built manually, instead use:
/// - [CoroutineBuilder]
/// - [node.start_coroutine](StartCoroutine::start_coroutine)
/// - [node.run_async_fn](StartCoroutine::start_async_fn)
#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct GodotCoroutine {
	pub(crate) base: Base<Node>,
	pub(crate) coroutine: Pin<Box<dyn Coroutine<(), Yield = Yield, Return = Variant>>>,
	pub(crate) poll_mode: PollMode,
	pub(crate) last_yield: Option<Yield>,
	pub(crate) paused: bool,
}

/// Builder struct for customizing coroutine behavior.
#[must_use]
pub struct CoroutineBuilder<T = ()> {
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
	pub(crate) on_finish: Vec<Callable>,
	/// Type hint for the coroutine's return value.
	pub(crate) type_hint: PhantomData<T>,
}

/// Defines whether the coroutine polls on process or physics frames. 
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PollMode {
	Process,
	Physics,
}

fn coroutine_with_variant_return<T: ToGodot>(
	f: impl Coroutine<(), Yield = Yield, Return = T>,
) -> impl Coroutine<(), Yield = Yield, Return = Variant> {
	#[coroutine] move || {
		let mut pin = Box::pin(f);
		loop {
			match pin.as_mut().resume(()) {
				CoroutineState::Yielded(_yield) => { yield _yield; }
				CoroutineState::Complete(result) => { return result.to_variant(); }
			}
		}
	}
}

impl<T: ToGodot> CoroutineBuilder<T> {
	/// Creates a new coroutine builder with default settings.
	pub const fn new(owner: Gd<Node>) -> Self {
		Self {
			owner,
			poll_mode: PollMode::Process,
			process_mode: ProcessMode::INHERIT,
			auto_start: true,
			on_finish: Vec::new(),
			type_hint: PhantomData,
		}
	}

	/// Whether the coroutine should be started automatically.
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
	pub fn on_finish(
		self,
		f: impl Fn(T) + Send + Sync + 'static,
	) -> Self 
		where T: FromGodot
	{
		self.callable_on_finish(
			Callable::from_fn("on_finish_call",
				move |args| {
					let conversion_result = 
						args.first()
							.ok_or_else(|| ConvertError::new("Args array is empty"))
							.and_then(|var| var.try_to::<T>())
							.map_err(|err| {
								godot_error!("{err}");
							})?;
					
					f(conversion_result);
					Ok(Variant::nil())
				})
		)
	}

	/// Adds `callable` to the list of callables that will be invoked when the coroutine finishes.
	///
	/// The return value of the coroutine(`T`) will be passed to `callable`.
	pub fn callable_on_finish(self, callable: Callable) -> Self {
		let mut on_finish = self.on_finish;
		on_finish.push(callable);
		Self {
			on_finish,
			..self
		}
	}

	/// Completes the builder, spawning the coroutine executor.
	///
	/// The executor is a node that will be added as a child of `owner`.
	pub fn spawn(
		self,
		f: impl Coroutine<(), Yield = Yield, Return = T> + 'static,
	) -> Gd<GodotCoroutine> 
		where T: 'static
	{
		let f = coroutine_with_variant_return(f);

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

		for callable in self.on_finish {
			coroutine.connect(SIGNAL_FINISHED.into(), callable);
		}

		let mut owner = self.owner;
		owner.add_child(coroutine.clone().upcast());

		coroutine
	}
}

pub trait StartCoroutine {
	/// Starts a new coroutine with default settings.
	///
	/// # Example
	///
	/// ```
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// let node = Node::new_alloc();
	///
	/// node.start_coroutine(
	///         #[coroutine] || {
	///             yield frames(5);
	///
	///             godot_print!("Profit :3");
	///         });
	///
	/// #[derive(GodotClass)]
	/// #[class(init, base = Node2D)]
	/// struct MyClass;
	///
	/// #[godot_api]
	/// impl MyClass {
	///     #[func]
	///     fn do_some_stuff(&self) {
	///         node.start_coroutine(
	///             #[coroutine] || {
	///                 yield frames(5);
	///
	///                 godot_print!("Profit :3");
	///             });
	///     }
	/// }
	/// ```
	fn start_coroutine<T: ToGodot + 'static>(
		&self,
		f: impl Coroutine<(), Yield = Yield, Return = T> + 'static,
	) -> Gd<GodotCoroutine> {
		self.coroutine().spawn(f)
	}

	#[cfg(feature = "async")]
	/// Works similarly to [start_coroutine], except it accepts an [async function](Future) as input.
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
	/// node.start_async_fn(
	///         async {
	///             smol::Timer::after(std::time::Duration::from_secs(5)).await;
	///             godot_print!("Profit :3"); 
	///         });
	/// 
	/// #[derive(GodotClass)]
	/// #[class(init, base = Node2D)]
	/// struct MyClass;
	/// 
	/// #[godot_api]
	/// impl MyClass {
	///     #[func]
	///     fn do_some_stuff(&self) {
	///         self.start_async_fn(
	///             async {
	///                 smol::Timer::after(std::time::Duration::from_secs(5)).await;
	///                 godot_print!("Profit :3"); 
	///             });
	///    }
	/// }
	/// ```
	fn start_async_fn<T: ToGodot + Send + 'static>(
		&self,
		f: impl std::future::Future<Output = T> + Send + 'static,
	) -> Gd<GodotCoroutine> {
		self.coroutine().spawn_async_fn(f)
	}

	fn coroutine<T: ToGodot>(&self) -> CoroutineBuilder<T>;
}

/// Anything that inherits Node can start coroutines.
impl<T: GodotClass + Inherits<Node>> StartCoroutine for Gd<T> {
	fn coroutine<TRet: ToGodot>(&self) -> CoroutineBuilder<TRet> {
		CoroutineBuilder::new(self.clone().upcast())
	}
}

impl<'a, T: WithBaseField + GodotClass<Base: Inherits<Node>>> StartCoroutine for &'a T {
	fn coroutine<TRet: ToGodot>(&self) -> CoroutineBuilder<TRet> {
		CoroutineBuilder::new(self.base().clone().upcast())
	}
}

impl<'a, T: WithBaseField + GodotClass<Base: Inherits<Node>>> StartCoroutine for &'a mut T {
	fn coroutine<TRet: ToGodot>(&self) -> CoroutineBuilder<TRet> {
		CoroutineBuilder::new(self.base().clone().upcast())
	}
}

const SIGNAL_FINISHED: &str = "finished";

#[godot_api]
impl GodotCoroutine {
	#[signal]
	fn finished(result: Variant) {}
	
	#[func]
	pub fn resume(&mut self) {
		self.paused = false;
	}

	#[func]
	pub fn pause(&mut self) {
		self.paused = true;
	}

	/// Forces the coroutine to finish immediately.
	///
	/// Does not trigger the `finished` signal, the result is returned directly.
	///
	/// Be careful, running all the instructions in a coroutine can lead to unexpected results.
	#[func]
	pub fn run_to_completion(&mut self) -> Variant {
		let mut iters_remaining = 4096;

		loop {
			match self.coroutine.as_mut().resume(()) {
				CoroutineState::Yielded(_) => {} // keep going
				CoroutineState::Complete(result) => {
					self.de_spawn();
					return result;
				}
			}

			iters_remaining -= 1;
			if iters_remaining == 0 {
				godot_error!(
					"The coroutine exceeded the maximum number of iterations(4096). \n\
					 This is likely a infinite loop, force stopping the coroutine.");
				return Variant::nil();
			}
		}
	}

	/// De-spawns the coroutine without triggering the `finished` signal
	#[func]
	pub fn kill(&mut self) {
		self.de_spawn();
	}

	/// De-spawns the coroutine and triggers the `finished` signal with `result` as the argument
	#[func]
	pub fn finish_with(&mut self, result: Variant) {
		self.base_mut().emit_signal(SIGNAL_FINISHED.into(), &[result]);
		self.de_spawn();
	}
}

#[godot_api]
impl INode for GodotCoroutine {
	fn process(&mut self, delta: f64) {
		if !self.paused && self.poll_mode == PollMode::Process {
			self.run(delta);
		}
	}

	fn physics_process(&mut self, delta: f64) {
		if !self.paused && self.poll_mode == PollMode::Physics {
			self.run(delta);
		}
	}
}

impl GodotCoroutine {
	fn de_spawn(&mut self) {
		let mut base = self.base().to_godot();

		if let Some(mut parent) = base.get_parent() {
			parent.remove_child(base.clone())
		}

		base.queue_free();
	}

	fn run(&mut self, delta_time: f64) {
		if let Some(result) = self.poll(delta_time) {
			self.finish_with(result);
		}
	}
	
	fn poll(&mut self, delta_time: f64) -> Option<Variant> {
		match &mut self.last_yield {
			Some(Yield::Frames(frames)) => {
				if *frames > 0 {
					*frames -= 1;
					None
				} else {
					self.last_yield = None;
					self.poll(delta_time)
				}
			}
			Some(Yield::Seconds(seconds)) => {
				if *seconds > delta_time {
					*seconds -= delta_time;
					None
				} else {
					let seconds = *seconds; // Deref needed to un-borrow self.last_yield
					self.last_yield = None;
					self.poll(delta_time - seconds)
				}
			}
			Some(Yield::Dyn(dyn_yield)) => {
				if dyn_yield.keep_waiting() {
					None
				} else {
					self.last_yield = None;
					self.poll(delta_time)
				}
			}
			None => {
				match self.coroutine.as_mut().resume(()) {
					CoroutineState::Yielded(next_yield) => {
						self.last_yield = Some(next_yield);
						self.poll(delta_time)
					}
					CoroutineState::Complete(result) => {
						Some(result)
					}
				}
			}
		}
	}
}

pub trait IsRunning {
	fn is_running(&self) -> bool;
}

impl IsRunning for Gd<GodotCoroutine> {
	/// Coroutines auto-destroy themselves when they finish
	fn is_running(&self) -> bool {
		self.is_instance_valid() && !self.bind().base().is_queued_for_deletion()
	}
}

pub trait IsFinished {
	fn is_finished(&self) -> bool;
}

impl<T: IsRunning> IsFinished for T {
	fn is_finished(&self) -> bool {
		!self.is_running()
	}
}