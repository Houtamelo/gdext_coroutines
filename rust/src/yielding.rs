use godot::prelude::*;

use crate::prelude::*;

/// Possible wait modes for coroutines.
/// 
/// See [frames], [seconds] and [KeepWaiting]
pub enum SpireYield {
	Frames(i64),
	Seconds(f64),
	Dyn(Box<dyn KeepWaiting>),
}

pub trait KeepWaiting {
	/// The coroutine calls this to check if it should keep waiting.
	/// 
	/// Execution will not resume as long as this returns true.
	/// 
	/// This will be polled on every [_process](INode::process) or [_physics_process](INode::physics_process), 
	/// depending on the configuration.
	fn keep_waiting(&mut self, delta_time: f64) -> bool;
}

impl<T: FnMut() -> bool> KeepWaiting for T {
	fn keep_waiting(&mut self, _delta_time: f64) -> bool {
		self()
	}
}

impl KeepWaiting for Gd<SpireCoroutine> {
	fn keep_waiting(&mut self, _delta_time: f64) -> bool {
		// Coroutines auto-destroy themselves when they finish
		self.is_instance_valid()
	}
}

pub trait WaitUntilFinished {
	fn wait_until_finished(&self) -> SpireYield;
}

impl WaitUntilFinished for Gd<SpireCoroutine> {
	fn wait_until_finished(&self) -> SpireYield {
		SpireYield::Dyn(Box::new(self.clone()))
	}
}

impl WaitUntilFinished for SpireCoroutine {
	fn wait_until_finished(&self) -> SpireYield {
		self.to_gd().wait_until_finished()
	}
}

/// Coroutine pauses execution as long as `f` returns true.
/// 
/// `f` is invoked whenever the coroutine is polled.
/// 
/// If un-paused, the coroutine is polled either on [_process](INode::process) 
/// or [_physics_process](INode::physics_process)
/// 
/// # Example
/// 
/// ```no_run
/// #![feature(coroutines)]
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn showcase_wait_while(node: Gd<Node>, message: AtomicBool) {
///      node.start_coroutine(
///           #[coroutine] move || {
///                yield wait_while(move || message.load(Ordering::Relaxed));
///                godot_print!("Message is no longer true! Resuming...");
///           });
/// }
///
/// ```
pub fn wait_while(f: impl FnMut() -> bool + 'static) -> SpireYield {
	SpireYield::Dyn(Box::new(f))
}

/// Coroutine resumes execution once `f` returns true.
/// 
/// `f` is invoked whenever the coroutine is polled. 
/// 
/// If un-paused, the coroutine is polled either on [process](INode::process) 
/// or [physics_process](INode::physics_process))
///
/// # Example
///
/// ```no_run
/// #![feature(coroutines)]
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn showcase_wait_until(node: Gd<Node>, message: AtomicBool) {
///      node.start_coroutine(
///           #[coroutine] move || {
///                yield wait_until(move || message.load(Ordering::Relaxed));
///                godot_print!("Message is true! Resuming...");
///           });
/// }
///
/// ```
pub fn wait_until(mut f: impl FnMut() -> bool + 'static) -> SpireYield {
	SpireYield::Dyn(Box::new(move || !f()))
}

/// Yield for a number of frames.
/// 
/// A frame equals a single [process](INode::process) 
/// or [physics_process](INode::physics_process)) call, depending on the coroutine's [PollMode].
///
/// # Example
///
/// ```no_run
/// #![feature(coroutines)]
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn showcase_frames(node: Gd<Node>) {
///      node.start_coroutine(
///           #[coroutine] move || {
///                yield frames(5);
///                godot_print!("5 frames have passed! Resuming...");
///           });
/// }
///
/// ```
pub const fn frames(frames: i64) -> SpireYield {
	SpireYield::Frames(frames)
}

/// Yield for a specific amount of engine time.
/// 
/// The time counter is affected by [Engine::time_scale](Engine::get_time_scale)
/// 
/// The time counter is also dependent on the coroutine's [PollMode].
/// 
/// Time does not pass if the coroutine's not being processed.
///
/// # Example
///
/// ```no_run
/// #![feature(coroutines)]
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn showcase_seconds(node: Gd<Node>) {
///      node.start_coroutine(
///           #[coroutine] move || {
///                yield seconds(7.5);
///                godot_print!("7.5 seconds have passed! Resuming...");
///           });
/// }
///
/// ```
pub const fn seconds(seconds: f64) -> SpireYield {
	SpireYield::Seconds(seconds)
}