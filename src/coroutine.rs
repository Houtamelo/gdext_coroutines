use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

use godot::classes::node::ProcessMode;
use godot::prelude::*;

use crate::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct GodotCoroutine {
	base: Base<Node>,
	coroutine: Pin<Box<dyn Coroutine<(), Yield = Yield, Return = ()>>>,
	last_yield: Option<Yield>,
	paused: bool,
}

#[must_use]
pub struct CoroutineBuilder {
	owner: Gd<Node>,
	process_mode: ProcessMode,
	auto_start: bool,
}

impl CoroutineBuilder {
	pub const fn new(owner: Gd<Node>) -> Self {
		Self {
			owner,
			process_mode: ProcessMode::INHERIT,
			auto_start: true,
		}
	}
	
	pub fn auto_start(self, auto_start: bool) -> Self {
		Self {
			auto_start,
			..self
		}
	}
	
	pub fn process_mode(self, process_mode: ProcessMode) -> Self {
		Self {
			process_mode,
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
					last_yield: None,
					paused: !self.auto_start,
				}
			});
		
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
	pub fn stop(&mut self) {
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
		if self.paused {
			return;
		}
		
		let is_finished = self.poll(delta);
		if is_finished {
			self.stop();
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