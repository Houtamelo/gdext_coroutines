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

mod coroutine;
mod yielding;
mod builder;
mod closure_types;

pub mod prelude {
	pub use crate::coroutine::{
		SpireCoroutine, 
		StartCoroutine,
		SIGNAL_FINISHED,
		IsRunning,
		IsFinished,
		PollMode,
	};
	
	pub use crate::builder::CoroutineBuilder;
	
	pub use crate::yielding::{
		seconds,
		frames, 
		wait_while, 
		wait_until, 
		KeepWaiting, 
		WaitUntilFinished,
		SpireYield as Yield,
	};
}