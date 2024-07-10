#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(unboxed_closures)]
#![cfg_attr(feature = "async", feature(async_fn_traits))]

#![allow(clippy::needless_return)]
#![allow(clippy::useless_conversion)]
#![allow(unused_doc_comments)]
#![warn(clippy::missing_const_for_fn)]

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod coroutine;
mod yielding;

#[cfg(feature = "async")]
mod async_impl;

#[cfg(feature = "integration_tests")] 
mod integration_tests;

pub mod prelude {
	pub use crate::coroutine::*;
	pub use crate::yielding::*;
}