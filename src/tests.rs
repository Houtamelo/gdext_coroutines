use godot::classes::node::ProcessMode;
use godot::prelude::*;

use crate::prelude::*;

fn test(mut node: Gd<Node>) {
	node.start_coroutine(
		#[coroutine] || {
			yield frames(2);

			yield seconds(5.0);

			let mut i = 0;

			yield wait_until(
				move || {
					i += 1;
					i >= 10
				}
			);

			let mut i = 0;

			yield wait_while(
				move || {
					i += 1;
					i < 10
				}
			);
		});
	
	node.build_coroutine()
	    .auto_start(false)
	    .process_mode(ProcessMode::ALWAYS)
	    .spawn(
		    #[coroutine] || {
			    yield frames(3);
			    //...
		    });
	
	let coroutine_ref = 
		node.start_coroutine(
			#[coroutine] || {
				yield seconds(2.0);
			});
	
	if coroutine_ref.is_running() {
		println!("Coroutine is running!");
	}
	
	if coroutine_ref.is_finished() {
		println!("Coroutine is finished!");
	}
	
	let mut coroutine_bind = coroutine_ref.bind_mut();
	coroutine_bind.resume();
	coroutine_bind.pause();
	coroutine_bind.stop();
	
	node.start_coroutine(
		#[coroutine] move || {
			yield coroutine_ref.wait_until_finished();
			
			println!("Coroutine finished!");
		});
}