use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;

mod shared;
mod window;
mod result;
mod plugins;
mod preferences;

use gio::prelude::*;

use window::App;
use plugins::Plugins;



fn main() {
	let mut plugins = Plugins::new();
	unsafe {
		plugins.load("/home/auri/Code/Projects/Scout/target/debug/libscout_program.so").expect("Invocation Failed");
		plugins.call("program", &[]).expect("Invocation double failed.");
	}

	let app = gtk::Application::new(Some("com.aurailus.scout"), Default::default())
		.expect("Failed to initialize GTK application.");

	app.connect_startup(|app| { App::new(&app); });
	app.run(&[]);
}
