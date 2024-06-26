# gdext_coroutines
"Run Rust coroutines in Godot 4.2+ (through GDExtension), inspired on Unity's Coroutines design."

# Beware
This crate uses 2 nightly(unstable) features:
```rs
#![feature(coroutines)]
#![feature(coroutine_trait)]
```

# Setup
Add the dependency to your Cargo.toml file:
```toml
[dependencies]
gdext_coroutines = "0.1.0"
```
Done :)

# What does this do?
Allows you to execute code in an asynchronous manner, the coroutines of this crate work very much like Unity's:

```rs
use gdext_coroutines::prelude::*;

let mut node: Gd<Label> = ..;
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
```

You can also configure the behavior of the coroutine:
```rs
let coroutine: Gd<Coroutine> =
    node.build_coroutine()
        // does not start automatically upon spawning
        .auto_start(false)
        // runs regardless of it's owner's process mode
        .process_mode(ProcessMode::ALWAYS)
        // Coroutine is polled in _physics_process instead of _process
        .poll_mode(PollMode::Physics)
        // creates the coroutine object(node) as a child of `node`, although the coroutine function won't automatically run since `auto_start` == `false`
        .spawn(
            #[coroutine] || {
                yield frames(69);
                godot_print!("Nice.");
            });

node.start_coroutine(
    #[coroutine] move || {
        godot_print!("Waiting until first coroutine finishes...");

        // You can also use coroutines as yields
        yield coroutine.wait_until_finished();

        godot_print!("First coroutine finished!");
    });

// When accessing the couroutine through `Gd<Coroutine>`, you can safely check if the coroutine is still "alive", this won't cause errors even if the coroutine has despawned.
// Note that the same is not valid for calling `bind()`/`bind_mut()`, since those require that the instance is still alive.
if coroutine.is_running() {
    godot_print!("Coroutine is running!");
}

if coroutine.is_finished() {
    godot_print!("Coroutine is finished!");
}

// Methods for controlling the coroutine.
let mut coroutine_bind = coroutine.bind_mut();
coroutine_bind.resume();
coroutine_bind.pause();
coroutine_bind.kill();
```

# How does this do?
A Coroutine is a struct that derives `Node`:
```rs
#[derive(GodotClass)]
#[class(no_init, base = Node)]
pub struct GodotCoroutine {
	base: Base<Node>,
	coroutine: Pin<Box<dyn Coroutine<(), Yield = Yield, Return = ()>>>,
	poll_mode: PollMode,
	last_yield: Option<Yield>,
	paused: bool,
}
```

When you call `spawn()` or `start_coroutine()`, a `GodotCoroutine` node is created, then added as a child of the caller:
```rs
pub fn spawn(
	self, 
	f: impl Coroutine<(), Yield = Yield, Return = ()> + 'static,
) -> Gd<GodotCoroutine> {
    let mut coroutine =
        Gd::from_init_fn(|base| {
            GodotCoroutine {
                base,
                coroutine: Box::pin(f),
                poll_mode: self.poll_mode,
                last_yield: None,
                paused: !self.auto_start,
            }
        });
    
    coroutine.set_process_priority(256);
    coroutine.set_physics_process_priority(256);
    
    coroutine.set_process_mode(self.process_mode);

    let mut owner = self.owner;
    owner.add_child(coroutine.clone().upcast());

    coroutine
}
```

Then every frame the `GodotCoroutine` polls the current yield to advance it's inner function.
```rs
#[godot_api]
impl INode for GodotCoroutine {
	fn process(&mut self, delta: f64) {
		match self.poll_mode {
			PollMode::Process => {}
			PollMode::Physics => {
				return;
			}
		}
		
		if self.paused {
			return;
		}
		
		let is_finished = self.poll(delta);
		if is_finished {
			self.kill();
		}
	}

	fn physics_process(&mut self, delta: f64) {
		match self.poll_mode {
			PollMode::Process => {
				return;
			}
			PollMode::Physics => {}
		}

		if self.paused {
			return;
		}

		let is_finished = self.poll(delta);
		if is_finished {
			self.kill();
		}
	}
}
```

It automatically destroys itself after finishing (`kill()` is called when `poll` returns true):
```rs
#[func]
pub fn kill(&mut self) {
    let mut base = self.base().to_godot();

    if let Some(mut parent) = base.get_parent() {
        parent.remove_child(base.clone())
    }

    base.queue_free();
}
```

And that's it.

---

### Also
You can make your own custom types of yields, just implement the trait `KeepWaiting`:
```rs
pub trait KeepWaiting {
	/// The coroutine calls this to check if it should keep waiting
	fn keep_waiting(&mut self) -> bool;
}
```

Then you can use that trait like this:
```rs
let my_custom_yield: dyn KeepWaiting = ...;

yield Yield::Dyn(Box::new(my_custom_yield));
```
