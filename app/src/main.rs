use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;

use gio::prelude::*;

mod window;
mod plugins;
mod preferences;

use window::App;
use plugins::PluginParser;

fn main() {
	let app = App::new();
	let mut plugins = PluginParser::new();
	plugins.load(app.clone(), "target/debug/libscout_plugin_application.so").expect("Invocation Failed");
	plugins.load(app.clone(), "target/debug/libscout_plugin_directory.so").expect("Invocation Failed");

	let gtk = gtk::Application::new(Some("com.aurailus.scout"), Default::default())
		.expect("Failed to initialize GTK application.");

	let app_clone = app.clone();
	gtk.connect_startup(move |gtk| {
		let bind_clone = app_clone.clone();
		app_clone.borrow_mut().init(bind_clone, gtk);
	});
	gtk.run(&[]);
}
