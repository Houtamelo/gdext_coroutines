use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

use godot::classes::node::ProcessMode;
use godot::prelude::*;

use crate::prelude::*;

/// The Godot class responsible for managing a coroutine.
/// This should not be built manually, instead use [CoroutineBuilder] or [node.start_coroutine](StartCoroutine::start_coroutine).
#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct GodotCoroutine {
	base: Base<Node>,
	coroutine: Pin<Box<dyn Coroutine<(), Yield = Yield, Return = ()>>>,
	poll_mode: PollMode,
	last_yield: Option<Yield>,
	paused: bool,
}

/// Builder struct for customizing coroutine behavior.
#[must_use]
pub struct CoroutineBuilder {
	owner: Gd<Node>,
	poll_mode: PollMode,
	process_mode: ProcessMode,
	/// Whether the coroutine should be started automatically.
	auto_start: bool,
}

/// Defines whether the coroutine polls on process or physics frames. 
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PollMode {
	Process,
	Physics,
}

impl CoroutineBuilder {
	pub const fn new(owner: Gd<Node>) -> Self {
		Self {
			owner,
			poll_mode: PollMode::Process,
			process_mode: ProcessMode::INHERIT,
			auto_start: true,
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
	
	/// See [PollMode]
	pub fn poll_mode(self, poll_mode: PollMode) -> Self {
		Self {
			poll_mode,
			..self
		}
	}
	
	pub fn spawn(
		self, 
		f: impl Coroutine<(), Yield = Yield, Return = ()> + 'static,
	) -> Gd<GodotCoroutine> {
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

pub trait StartCoroutine {
	/// Starts a new coroutine with default settings
	fn start_coroutine(
		&mut self,
		f: impl Coroutine<(), Yield = Yield, Return = ()> + 'static,
	) -> Gd<GodotCoroutine> {
		self.build_coroutine().spawn(f)
	}
	
	fn build_coroutine(&mut self) -> CoroutineBuilder;
}

/// Anything that inherits Node can start coroutines
impl<T: Inherits<Node>> StartCoroutine for Gd<T> {
	fn build_coroutine(&mut self) -> CoroutineBuilder {
		CoroutineBuilder::new(self.clone().upcast())
	}
}

#[godot_api]
impl GodotCoroutine {
	#[func]
	pub fn resume(&mut self) {
		self.paused = false;
	}
	
	#[func]
	pub fn pause(&mut self) {
		self.paused = true;
	}
	
	#[func]
	pub fn kill(&mut self) {
		let mut base = self.base().to_godot();

		if let Some(mut parent) = base.get_parent() {
			parent.remove_child(base.clone())
		}

		base.queue_free();
	}
}

#[godot_api]
impl INode for GodotCoroutine {
	fn process(&mut self, delta: f64) {
		match self.poll_mode {
			PollMode::Process => {}
			PollMode::Physics => {
				return;
			}
		}
		
		if self.paused {
			return;
		}
		
		let is_finished = self.poll(delta);
		if is_finished {
			self.kill();
		}
	}

	fn physics_process(&mut self, delta: f64) {
		match self.poll_mode {
			PollMode::Process => {
				return;
			}
			PollMode::Physics => {}
		}

		if self.paused {
			return;
		}

		let is_finished = self.poll(delta);
		if is_finished {
			self.kill();
		}
	}
}

impl GodotCoroutine {
	/// Returns true if the coroutine has finished running.
	fn poll(&mut self, delta_time: f64) -> bool {
		match &mut self.last_yield {
			Some(Yield::Frames(frames)) => {
				if *frames > 0 {
					*frames -= 1;
					false
				} else {
					self.last_yield = None;
					self.poll(delta_time)
				}
			}
			Some(Yield::Seconds(seconds)) => {
				if *seconds > delta_time {
					*seconds -= delta_time;
					false
				} else {
					let seconds = *seconds; // Deref needed to un-borrow self.last_yield
					self.last_yield = None;
					self.poll(delta_time - seconds)
				}
			}
			Some(Yield::Dyn(dyn_yield)) => {
				if dyn_yield.keep_waiting() {
					false
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
					CoroutineState::Complete(_) => {
						true
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
		self.is_instance_valid()
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