#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(unboxed_closures)]
#![cfg_attr(feature = "async", feature(async_fn_traits))]

#![allow(clippy::needless_return)]
#![allow(clippy::useless_conversion)]
#![allow(unused_doc_comments)]
#![allow(private_bounds)]

#![doc = include_str!("../../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use godot::builtin::{Callable, Variant};

mod coroutine;
mod yielding;
mod builder;
mod start_coroutine;

#[cfg(feature = "async")]
mod start_async_task;

pub(crate) enum OnFinishCall {
	Closure(Box<dyn FnOnce(Variant)>),
	Callable(Callable),
}

pub mod prelude {
	pub use crate::coroutine::{
		SpireCoroutine,
		SIGNAL_FINISHED,
		IsRunning,
		IsFinished,
		IsPaused,
		PollMode,
	};

	pub use crate::yielding::{
		seconds,
		frames,
		wait_while,
		wait_until,
		KeepWaiting,
		WaitUntilFinished,
		SpireYield as Yield,
	};
	
	pub use crate::start_coroutine::StartCoroutine;
	pub use crate::builder::CoroutineBuilder;
	
	#[cfg(feature = "async")]
	pub use crate::start_async_task::StartAsyncTask;
}