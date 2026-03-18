use std::sync::{Arc, atomic::AtomicBool};

use godot::{obj::WithSignals, prelude::*, register::TypedSignal};

use crate::prelude::*;

/// Possible wait modes for coroutines.
///
/// See [frames], [seconds] and [KeepWaiting]
pub enum SpireYield {
    Frames(i64),
    Seconds(f64),
    Coroutine(Gd<SpireCoroutine>),
    Signal {
        signal: Signal,
        emission_tracker: Option<Arc<AtomicBool>>,
    },
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
    fn keep_waiting(&mut self, _delta_time: f64) -> bool { self() }
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
    fn wait_until_finished(&self) -> SpireYield { SpireYield::Coroutine(self.clone()) }
}

impl WaitUntilFinished for SpireCoroutine {
    fn wait_until_finished(&self) -> SpireYield { self.to_gd().wait_until_finished() }
}

/// Coroutine pauses execution until the given [Signal] is emitted.
///
/// This is the untyped variant — accepts any `Signal` directly.
///
/// See [wait_for_signal] for the typed version.
///
/// # Example
///
/// ```no_run
/// #![feature(coroutines)]
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn showcase_wait_for_signal_untyped(node: Gd<Node>) {
///      let signal = Signal::from_object_signal(&node, "child_entered_tree");
///      node.start_coroutine(
///           #[coroutine] move || {
///                yield wait_for_signal_untyped(signal);
///                godot_print!("Signal emitted! Resuming...");
///           });
/// }
///
/// ```
pub fn wait_for_signal_untyped(signal: Signal) -> SpireYield {
    SpireYield::Signal {
        signal,
        emission_tracker: None,
    }
}

/// Coroutine pauses execution until the given typed signal is emitted.
///
/// Accepts a reference to a [TypedSignal], which can be obtained from
/// a Godot class via `gd.signals().signal_name()`.
///
/// See [wait_for_signal_untyped] for the untyped variant.
///
/// # Example
///
/// ```no_run
/// #![feature(coroutines)]
/// use gdext_coroutines::prelude::*;
/// use godot::prelude::*;
///
/// fn showcase_wait_for_signal(node: Gd<Node>) {
///      node.start_coroutine(
///           #[coroutine] move || {
///                let sig = node.signals().child_entered_tree();
///                yield wait_for_signal(&sig);
///                godot_print!("Signal emitted! Resuming...");
///           });
/// }
///
/// ```
pub fn wait_for_signal<C, PS>(signal: &TypedSignal<C, PS>) -> SpireYield
where
    C: WithSignals,
    PS: godot::meta::ParamTuple,
{
    let untyped = signal.to_untyped();
    wait_for_signal_untyped(untyped)
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
pub fn wait_while(f: impl FnMut() -> bool + 'static) -> SpireYield { SpireYield::Dyn(Box::new(f)) }

/// Coroutine resumes execution once `f` returns true.
///
/// `f` is invoked whenever the coroutine is polled.
///
/// If un-paused, the coroutine is polled either on [process](INode::process)
/// or [physics_process](INode::physics_process)
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
pub fn wait_until(mut f: impl FnMut() -> bool + 'static) -> SpireYield { SpireYield::Dyn(Box::new(move || !f())) }

/// Yield for a number of frames.
///
/// A frame equals a single [process](INode::process)
/// or [physics_process](INode::physics_process) call, depending on the coroutine's [PollMode].
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
pub const fn frames(frames: i64) -> SpireYield { SpireYield::Frames(frames) }

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
pub const fn seconds(seconds: f64) -> SpireYield { SpireYield::Seconds(seconds) }

/// For those who really hate words.
pub mod shortcuts {
    pub use super::wait_for_signal_untyped as signal_untyped;
    pub use super::wait_for_signal as signal;
    pub use super::wait_until as until;
    pub use super::wait_while as whilst;
    pub use next_frame as unity_null;
    
    #[inline]
    pub fn next_frame() -> super::SpireYield { super::frames(1) }
}