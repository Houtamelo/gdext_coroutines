# gdext_coroutines

"Run Rust coroutines and async code in Godot 4.2+ (through GDExtension), inspired on Unity's Coroutines design."

# Beware

This crate uses 5 nightly(unstable) features:

```rust
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(unboxed_closures)]
#![cfg_attr(feature = "async", feature(async_fn_traits))]
```

It also requires GdExtension's `experimental_threads` feature

# Setup

Add the dependency to your Cargo.toml file:

```toml
[dependencies]
gdext_coroutines = "0.7"
```

# What does this do?

Allows you to execute code in an asynchronous manner, the coroutines of this crate work very much like Unity's.

It also allows you to execute async code(futures), the implementation uses the crate `smol` and requires the feature `async`.

```rust ignore
#![feature(coroutines)]
use gdext_coroutines::prelude::*;
use godot::prelude::*;

fn run_some_routines(node: Gd<Label>) {
	node.start_coroutine(
		#[coroutine] || {
			godot_print!("Starting coroutine");
            
			godot_print!("Waiting for 5 seconds...");
			yield seconds(5.0);
			godot_print!("5 seconds have passed!");

			godot_print!("Waiting for 30 frames");
			yield frames(30);
			godot_print!("30 frames have passed!");

			godot_print!("Waiting until pigs start flying...");
			let pig: Gd<Node2D> = create_pig();
			yield wait_until(move || pig.is_flying());
			godot_print!("Wow! Pigs are now able to fly! Somehow...");
            
			godot_print!("Waiting while pigs are still flying...");
			let pig: Gd<Node2D> = grab_pig();
			yield wait_while(move || pig.is_flying());
			godot_print!("Finally, no more flying pigs, oof.");
		});

	node.start_async_task(
		async {
			godot_print!("Executing async code!");
			smol::Timer::after(Duration::from_secs(10)).await;
			godot_print!("Async function finished after 10 real time seconds!");
		});
}
```

For more examples, check the `integration_tests` folder in the repository.

---

# How does this do?

A Coroutine is a struct that derives `Node`
```rust ignore
#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct SpireCoroutine { /* .. */ }
```

When you invoke `start_coroutine()`, `start_async_task()`, or `spawn()`, a `SpireCoroutine` node is created, then added as a child of the caller.

Then, on every frame:
- Rust Coroutines(`start_coroutine`): polls the current yield to advance its inner function.
- Rust Futures(`start_async_task`): checks if the future has finished executing.

```rust ignore
#[godot_api]
impl INode for SpireCoroutine {
	fn process(&mut self, delta: f64) {
		if !self.paused && self.poll_mode == PollMode::Process {
			self.run(delta);
		}
	}

	fn physics_process(&mut self, delta: f64) {
		if !self.paused && self.poll_mode == PollMode::Physics {
			self.run(delta);
		}
	}
}
```

Then it automatically destroys itself after finishing:

```rust ignore
fn run(&mut self, delta_time: f64) {
	if let Some(result) = self.poll(delta_time) {
		self.finish_with(result);
	}
}

pub fn finish_with(&mut self, result: Variant) {
	/* .. */

	self.base_mut().emit_signal(SIGNAL_FINISHED.into(), &[result]);
	self.de_spawn();
}
```

Since the coroutine is a child node of whoever created it, the behavior is tied to its parent:
- If the parent exits the scene tree, the coroutine pauses running (since it requires `_process/_physics_process` to run).
- If the parent is queued free, the coroutine is also queued free, and its `finished` signal never triggers.

---

# Notes

### 1 - You can await coroutines from GdScript, using the signal `finished`
```js
var coroutine: SpireCoroutine = ..
var result = await coroutine.finished
```

`result` contains the return value of your coroutine/future.

---

### 2 - You can make your own custom types of yields, just implement the trait `KeepWaiting`
```rust ignore
pub trait KeepWaiting {
	/// The coroutine calls this to check if it should keep waiting
	fn keep_waiting(&mut self) -> bool;
}
```

Then you can use that trait like this:

```rust ignore
let my_custom_yield: dyn KeepWaiting = ...;

yield Yield::Dyn(Box::new(my_custom_yield));
```

### 3 - Your main crate must have at least one godot class defined in it
Otherwise, this crate's godot classes will not be registered in Godot.

This is a known issue in gdext-rust, it's not related to gdext-coroutines.