use std::alloc::System;
#[global_allocator]
static ALLOCATOR: System = System;

use gio::prelude::*;

mod app;
mod window;
mod plugin;
mod preferences;

use plugin::PluginParser;
use app::{ App, AppCallbacks };

fn main() {
	let app = App::new();
	let plugins = PluginParser::new();

	let plugins_clone = plugins.clone();
	app.borrow_mut().bind(AppCallbacks {
		on_search: Box::new(move |query| plugins_clone.borrow_mut().get_results(query))
	});

	plugins.borrow_mut().load(&app, "target/debug/libscout_plugin_application.so").expect("Invocation Failed");
	plugins.borrow_mut().load(&app, "target/debug/libscout_plugin_directory.so").expect("Invocation Failed");

	let gtk = gtk::Application::new(Some("com.aurailus.scout"), Default::default())
		.expect("Failed to initialize GTK application.");

	let app_clone = app.clone();
	gtk.connect_startup(move |gtk| {
		let bind = app_clone.clone();
		app_clone.borrow_mut().init(gtk, &bind);
	});
	gtk.run(&[]);
}
