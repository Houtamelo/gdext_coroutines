#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![feature(const_type_name)]
#![feature(hash_extract_if)]
#![feature(iter_array_chunks)]
#![feature(coroutines)]
#![feature(iter_from_coroutine)]
#![feature(coroutine_trait)]
#![allow(clippy::needless_return)]
#![allow(clippy::useless_conversion)]
#![allow(unused_doc_comments)]
#![warn(clippy::missing_const_for_fn)]

mod coroutine;
mod yielding;
mod tests;

struct Coroutines;

#[godot::prelude::gdextension]
unsafe impl godot::prelude::ExtensionLibrary for Coroutines {}

pub mod prelude {
	pub use crate::coroutine::*;
	pub use crate::yielding::*;
}