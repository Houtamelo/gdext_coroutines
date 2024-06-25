use godot::prelude::*;

use crate::prelude::*;

/// Enum that tells the coroutine how long it should wait
pub enum Yield {
	Frames(i64),
	Seconds(f64),
	Dyn(Box<dyn KeepWaiting>),
}

pub trait KeepWaiting {
	/// Returns true if the coroutine can continue executing.
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

/// Coroutine pauses execution as long as `f` returns true
/// `f` is invoked once on each `_process` call
pub fn wait_while(mut f: impl FnMut() -> bool + 'static) -> Yield {
	Yield::Dyn(Box::new(move || !f()))
}

/// Coroutine resumes execution once `f` returns true
/// `f` is invoked once on each `_process` call
pub fn wait_until(f: impl FnMut() -> bool + 'static) -> Yield {
	Yield::Dyn(Box::new(f))
}

pub fn frames(frames: i64) -> Yield {
	Yield::Frames(frames)
}

pub fn seconds(seconds: f64) -> Yield {
	Yield::Seconds(seconds)
}