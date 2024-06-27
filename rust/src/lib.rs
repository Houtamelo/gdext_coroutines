#![feature(coroutines)]
#![feature(coroutine_trait)]
#![allow(clippy::needless_return)]
#![allow(clippy::useless_conversion)]
#![allow(unused_doc_comments)]
#![warn(clippy::missing_const_for_fn)]

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod coroutine;
mod yielding;

#[cfg(feature = "integration_tests")] mod tests;

pub mod prelude {
	pub use crate::coroutine::*;
	pub use crate::yielding::*;
}