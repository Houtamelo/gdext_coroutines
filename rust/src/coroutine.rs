use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

use godot::obj::WithBaseField;
use godot::prelude::*;

use crate::closure_types::{OnFinishCall, PivotTrait};
use crate::prelude::*;
use crate::yielding::SpireYield;

/// A Godot class responsible for managing a coroutine.
///
/// This should not be built manually, instead use:
/// - [CoroutineBuilder]
/// - [node.start_coroutine](StartCoroutine::start_coroutine)
/// - [node.run_async_fn](StartCoroutine::start_async_fn)
#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct SpireCoroutine {
	pub(crate) base: Base<Node>,
	pub(crate) coroutine: Pin<Box<dyn Coroutine<(), Yield = SpireYield, Return = Variant>>>,
	pub(crate) poll_mode: PollMode,
	pub(crate) last_yield: Option<SpireYield>,
	pub(crate) paused: bool,
	pub(crate) calls_on_finish: Vec<OnFinishCall>,
}

/// Defines whether the coroutine polls on process or physics frames. 
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PollMode {
	Process,
	Physics,
}

pub trait StartCoroutine {
	/// Starts a new coroutine with default settings.
	///
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;
	///
	/// #[derive(GodotClass)]
	/// #[class(init, base = Node2D)]
	/// struct MyClass {
	///     base: Base<Node2D>,
	/// }
	///
	/// #[godot_api]
	/// impl MyClass {
	///     #[func]
	///     fn do_some_stuff(&self) {
	///         self.start_coroutine( // self can be replaced for Gd<T: Inherits<Node>>
	///             #[coroutine] || {
	///                 yield frames(5);
	///
	///                 godot_print!("Profit :3");
	///             });
	///
	///         #[cfg(feature = "async")]
	///         {
	///             self.start_coroutine( // self can be replaced for Gd<T: Inherits<Node>>
	///                 async {
	///                     let result = smol::fs::read_to_string("hello.txt").await;
	///                     return result.unwrap();
	///                 });
	///         }
	///     }
	/// }
	/// ```
	fn start_coroutine<Return: ToGodot, Marker>(
		&self,
		f: impl PivotTrait<Marker, Return>,
	) -> Gd<SpireCoroutine> {
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
	/// fn build_coroutine(node: Gd<Node2D>) {
	///      node.coroutine(
	///          #[coroutine] || {
	///              godot_print!("This is a customized coroutine!");
	///              
	///              return Array::from(&[20, 30, 50, 69]);
	///          })
	///          .auto_start(false)
	///          .process_mode(ProcessMode::WHEN_PAUSED)
	///          .spawn();
	/// }
	/// ```
	fn coroutine<Return: ToGodot, Marker>(
		&self,
		f: impl PivotTrait<Marker, Return>,
	) -> CoroutineBuilder<Return>;
}

impl<TSelf> StartCoroutine for Gd<TSelf>
	where
		TSelf: GodotClass + Inherits<Node>,
{
	fn coroutine<Return: ToGodot, Marker>(
		&self,
		f: impl PivotTrait<Marker, Return>,
	) -> CoroutineBuilder<Return> {
		CoroutineBuilder::new(self.clone().upcast(), f)
	}
}

impl<'a, T> StartCoroutine for &'a T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn coroutine<Return: ToGodot, Marker>(
		&self,
		f: impl PivotTrait<Marker, Return>,
	) -> CoroutineBuilder<Return> {
		CoroutineBuilder::new(self.base().clone().upcast(), f)
	}
}

impl<'a, T> StartCoroutine for &'a mut T
	where
		T: WithBaseField + GodotClass<Base: Inherits<Node>>,
{
	fn coroutine<Return: ToGodot, Marker>(
		&self,
		f: impl PivotTrait<Marker, Return>,
	) -> CoroutineBuilder<Return> {
		CoroutineBuilder::new(self.base().clone().upcast(), f)
	}
}

/// The name of the finished signal.
/// 
/// You can manually connect to this signal to get the coroutine's result when it finishes.
/// 
/// # Example
/// 
/// ```no_run
/// #![feature(coroutines)]
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn manually_connect(node: Gd<Node>) {
///     let mut coroutine = 
///         node.start_coroutine(
///             #[coroutine] || {
///                 yield seconds(2.0);
///                 
///                 return "Hello, I'm 2 seconds late!";
///             });
///      
///     coroutine.connect(SIGNAL_FINISHED.into(), Callable::from_fn("print_result", 
///         |args| {
///             let result = args.first().and_then(|var| var.try_to::<String>().ok()).unwrap();
///             assert_eq!(result.as_str(), "Hello, I'm 2 seconds late!");
///             Ok(Variant::nil())
///         }));
/// }
/// ```
pub const SIGNAL_FINISHED: &str = "finished";

#[godot_api]
impl SpireCoroutine {
	#[signal]
	fn finished(result: Variant) {}
	
	#[func]
	fn is_paused(&self) -> bool {
		self.paused
	}
	
	#[func]
	fn is_running(&self) -> bool {
		!self.paused && !self.base().is_queued_for_deletion()
	}
	
	#[func]
	fn is_finished(&self) -> bool {
		self.base().is_queued_for_deletion()
	}

	/// Resumes the coroutine.
	/// 
	/// Resuming a coroutine that's already running doesn't do anything.
	#[func]
	pub fn resume(&mut self) {
		self.paused = false;
	}

	/// Pauses the coroutine, ensuring it won't execute any instructions until it is resumed.
	/// 
	/// Pausing a coroutine that's already paused doesn't do anything.
	#[func]
	pub fn pause(&mut self) {
		self.paused = true;
	}

	/// Forces the coroutine to finish immediately.
	///
	/// Does not trigger the `finished` signal, the result is returned directly.
	///
	/// Be careful, running all the instructions in a coroutine at once can lead to unexpected results.
	#[func]
	pub fn force_run_to_completion(&mut self) -> Variant {
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

	/// De-spawns the coroutine.
	///
	/// Does not trigger the `finished` signal.
	#[func]
	pub fn kill(&mut self) {
		self.de_spawn();
	}

	/// De-spawns the coroutine.
	///
	/// Triggers the `finished` signal with `result` as the argument.
	#[func]
	pub fn finish_with(&mut self, result: Variant) {
		for call in self.calls_on_finish.drain(..) {
			match call {
				OnFinishCall::Closure(closure) => {
					closure(result.clone());
				}
				OnFinishCall::Callable(callable) => {
					if callable.is_valid() {
						callable.callv(VariantArray::from(&[result.clone()]));
					}
				}
			}
		}

		self.base_mut().emit_signal(SIGNAL_FINISHED.into(), &[result]);
		self.de_spawn();
	}
}

#[godot_api]
impl INode for SpireCoroutine {
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

impl SpireCoroutine {
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
			Some(SpireYield::Frames(frames)) => {
				if *frames > 0 {
					*frames -= 1;
					None
				} else {
					self.last_yield = None;
					self.poll(delta_time)
				}
			}
			Some(SpireYield::Seconds(seconds)) => {
				if *seconds > delta_time {
					*seconds -= delta_time;
					None
				} else {
					let seconds = *seconds; // Deref needed to un-borrow self.last_yield
					self.last_yield = None;
					self.poll(delta_time - seconds)
				}
			}
			Some(SpireYield::Dyn(dyn_yield)) => {
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

impl IsRunning for Gd<SpireCoroutine> {
	fn is_running(&self) -> bool {
		self.is_instance_valid() && self.bind().is_running()
	}
}

pub trait IsFinished {
	fn is_finished(&self) -> bool;
}

impl IsFinished for Gd<SpireCoroutine> {
	fn is_finished(&self) -> bool {
		!self.is_instance_valid() || self.bind().is_finished()
	}
}