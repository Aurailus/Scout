mod shared;
mod window;
mod result;
mod preferences;

use gio::prelude::*;

use window::App;

fn main() {
	let app = gtk::Application::new(Some("com.aurailus.scout"), Default::default())
		.expect("Failed to initialize GTK application.");

	// app.connect_activate(|app| {
	// 	// app.emit("activ")
	// 	app.register::<gio::Cancellable>(None).unwrap();
	// });
	app.connect_startup(|app| { App::new(&app); });
	app.run(&[]);
}
