use dirs;
use gio::prelude::*;

mod shared;
mod window;
mod fileinfo;

use window::App;

fn main() {
	std::env::set_current_dir(dirs::home_dir().unwrap()).unwrap();

	let app = gtk::Application::new(Some("com.aurailus.explorer"), Default::default())
		.expect("Failed to initialize GTK application.");

	app.connect_activate(|app| { app.register::<gio::Cancellable>(None).unwrap(); });
	app.connect_startup(|app| { App::new(app, &std::path::Path::new(".")); });
	app.run(&[]);
}
