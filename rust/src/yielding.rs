use godot::prelude::*;

use crate::prelude::*;

/// Possible wait modes for coroutines.
pub enum Yield {
	Frames(i64),
	Seconds(f64),
	Dyn(Box<dyn KeepWaiting>),
}

pub trait KeepWaiting {
	/// The coroutine calls this to check if it should keep waiting
	fn keep_waiting(&mut self) -> bool;
}

/// Predicates as yields
impl<T: FnMut() -> bool> KeepWaiting for T {
	fn keep_waiting(&mut self) -> bool {
		self()
	}
}

/// For chaining coroutines as yields
impl KeepWaiting for Gd<GodotCoroutine> {
	fn keep_waiting(&mut self) -> bool {
		// Coroutines auto-destroy themselves when they finish
		self.is_instance_valid()
	}
}

pub trait WaitUntilFinished {
	fn wait_until_finished(&self) -> Yield;
}

impl WaitUntilFinished for Gd<GodotCoroutine> {
	fn wait_until_finished(&self) -> Yield {
		Yield::Dyn(Box::new(self.clone()))
	}
}

impl WaitUntilFinished for GodotCoroutine {
	fn wait_until_finished(&self) -> Yield {
		self.to_gd().wait_until_finished()
	}
}

/// Coroutine pauses execution as long as `f` returns true.
/// 
/// `f` is invoked whenever the coroutine is polled. 
/// 
/// If un-paused, the coroutine is polled either on [_process](INode::process) or [_physics_process]
/// (INode::physics_process)
pub fn wait_while(f: impl FnMut() -> bool + 'static) -> Yield {
	Yield::Dyn(Box::new(f))
}

/// Coroutine resumes execution once `f` returns true.
/// 
/// `f` is invoked whenever the coroutine is polled. (Either on [process](INode::process) or [physics_process](INode::physics_process))
pub fn wait_until(mut f: impl FnMut() -> bool + 'static) -> Yield {
	Yield::Dyn(Box::new(move || !f()))
}

/// Yield for a number of frames.
/// 
/// The frame counter is dependent on the coroutine's [PollMode].
pub const fn frames(frames: i64) -> Yield {
	Yield::Frames(frames)
}

/// Yield for a specific amount of engine time. (The time counter is affected by [Engine::time_scale](Engine::get_time_scale))
/// 
/// The time counter is also dependent on the coroutine's [PollMode].
pub const fn seconds(seconds: f64) -> Yield {
	Yield::Seconds(seconds)
}