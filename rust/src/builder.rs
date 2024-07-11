use godot::classes::node::ProcessMode;
use godot::prelude::*;
use crate::closure_types::{CoroutinePivot, OnFinishCall, OnFinishPivot, PivotTrait};
use crate::prelude::*;

/// Builder struct for customizing coroutine behavior.
#[must_use]
pub struct CoroutineBuilder<TReturn = ()> {
	pub(crate) f_pivot: CoroutinePivot,
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
	pub(crate) type_hint: std::marker::PhantomData<TReturn>,
}

impl<Return: ToGodot> CoroutineBuilder<Return> {
	/// Creates a new coroutine builder with default settings.
	pub fn new<Marker>(
		owner: Gd<Node>,
		f: impl PivotTrait<Marker, Return>, 
	) -> Self {
		Self {
			f_pivot: f.into_pivot(),
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
	/// # Example
	///
	/// ```no_run
	/// #![feature(coroutines)]
	/// use godot::prelude::*;
	/// use gdext_coroutines::prelude::*;	///
	/// 
	/// fn showcase_finish(node: Gd<Node2D>) {
	///     node.coroutine(
	///         #[coroutine] || { 
	///             yield frames(2);
	///             return 5;
	///         })
	///         .on_finish(|res| {
	///             println!("Coroutine finished, result: {res}");
	///         })
	///         .spawn();
	/// }
	/// ```
	pub fn on_finish(
		self,
		f: impl Into<OnFinishPivot<Return>>,
	) -> Self
		where
			Return: FromGodot,
			Return: 'static,
			Return: Send,
	{
		let mut calls_on_finish = self.calls_on_finish;
		calls_on_finish.push(OnFinishCall::from(f.into()));

		Self {
			calls_on_finish,
			..self
		}
	}

	/// Completes the builder, spawning the coroutine's executor.
	///
	/// The executor is a node that will be added as a child of `owner`.
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
	///                 
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
					coroutine: self.f_pivot.f,
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
		owner.add_child(coroutine.clone().upcast());

		coroutine
	}
}