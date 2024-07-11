use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;

use godot::prelude::*;
use crate::yielding::SpireYield;

pub struct CoroutineType;

pub(crate) struct CoroutinePivot {
	pub(crate) f: Pin<Box<dyn Coroutine<(), Yield = SpireYield, Return = Variant>>>,
}

pub(crate) trait PivotTrait<Marker, Var> {
	fn into_pivot(self) -> CoroutinePivot; 
}

impl<T, F> PivotTrait<CoroutineType, T> for F
	where
		T: ToGodot,
		F: 'static, 
		F: Coroutine<(), Yield = SpireYield, Return = T>,
{
	fn into_pivot(self) -> CoroutinePivot {
		let routine =
			#[coroutine] move || {
				let mut pin = Box::pin(self);
				loop {
					match pin.as_mut().resume(()) {
						CoroutineState::Yielded(_yield) => { yield _yield; }
						CoroutineState::Complete(result) => { return result.to_variant(); }
					}
				}
			};

		CoroutinePivot { f: Box::pin(routine) }
	}
}

#[cfg(feature = "async")]
mod async_impl {
	use std::future::Future;
	use crate::prelude::*;
	use super::*;

	pub struct FutureType;

	impl<T, F> PivotTrait<FutureType, T> for F
		where
			T: ToGodot + Send + 'static,
			F: Future<Output = T>,
			F: Send,
			F: 'static,
	{
		fn into_pivot(self) -> CoroutinePivot {
			let task = smol::spawn(self);

			let routine =
				#[coroutine] move || {
					while !task.is_finished() {
						yield frames(1);
					}

					smol::block_on(task).to_variant()
				};

			CoroutinePivot { f: Box::pin(routine) }
		}
	}
}

pub(crate) enum OnFinishPivot<T> {
	Closure(Box<dyn FnOnce(T)>),
	Callable(Callable),
}

pub(crate) enum OnFinishCall {
	Closure(Box<dyn FnOnce(Variant)>),
	Callable(Callable),
}

impl<T, F> From<F> for OnFinishPivot<T>
	where
		T: 'static,
		F: 'static + FnOnce(T),
{
	fn from(value: F) -> Self {
		Self::Closure(Box::new(value))
	}
}

impl<T> From<Callable> for OnFinishPivot<T> {
	fn from(value: Callable) -> Self {
		Self::Callable(value)
	}
}

impl From<Callable> for OnFinishCall {
	fn from(value: Callable) -> Self {
		Self::Callable(value)
	}
}

impl<T: 'static + FromGodot + Send> From<OnFinishPivot<T>> for OnFinishCall {
	fn from(f: OnFinishPivot<T>) -> Self {
		match f.into() {
			OnFinishPivot::Closure(closure) => {
				OnFinishCall::Closure(Box::new(|var| {
					match var.try_to::<T>() {
						Ok(t) => {
							closure(t);
						}
						Err(err) => {
							godot_error!("{err}");
						}
					}
				}))
			}
			OnFinishPivot::Callable(callable) => OnFinishCall::Callable(callable),
		}
	}
}