use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;

mod shared;
mod window;
mod plugins;
mod preferences;

use gio::prelude::*;

use window::App;

fn main() {
	let app = gtk::Application::new(Some("com.aurailus.scout"), Default::default())
		.expect("Failed to initialize GTK application.");

	app.connect_startup(|app| { App::new(&app); });
	app.run(&[]);
}
