use gtk::prelude::*;
use std::os::unix::process::CommandExt;

use scout_core::Shared;
use crate::preferences::Preferences;

use super::style;
use super::prefs_page;

static WIDTH: i32 = 700;
static HEIGHT: i32 = 454;

pub struct PrefsWindow {
	notebook: gtk::Notebook
}

impl PrefsWindow {
	fn add_page<T: IsA<gtk::Widget>>(&self, label: &str, page: &T) {
		let label = gtk::Label::new(Some(label));
		label.set_xalign(1.0);
		self.notebook.append_page(page, Some(&label));
	}

	pub fn new(preferences: &Preferences) -> Shared<Self> {
		let preferences = Shared::new(preferences.clone());

		let window = gtk::Window::new(gtk::WindowType::Toplevel);
		window.set_widget_name("PreferencesDialog");
		window.set_icon_name(Some("preferences-other"));
		window.set_title("Scout Preferences");
		window.set_default_size(WIDTH, HEIGHT);
		window.set_resizable(false);
		style::style(&window, &preferences.borrow(), &vec![]);

		let header = gtk::HeaderBar::new();
		header.set_title(Some("Scout Preferences"));
		header.set_show_close_button(true);
		window.set_titlebar(Some(&header));

		let save_button = gtk::Button::with_label("Save");
		save_button.get_style_context().add_class("suggested-action");
		header.pack_start(&save_button);

		let overlay = gtk::Overlay::new();
		window.add(&overlay);

		let notebook = gtk::Notebook::new();
		notebook.set_tab_pos(gtk::PositionType::Left);
		notebook.set_widget_name("Preferences");
		notebook.set_show_border(false);
		overlay.add(&notebook);

		let preferences_clone = preferences.clone();
		save_button.connect_clicked(move |_| {
			preferences_clone.borrow().save().unwrap();
    	std::process::Command::new("/proc/self/exe").exec();
    	std::process::exit(0);
		});

	  let prefs = Shared::new(PrefsWindow { notebook });

		prefs.borrow().add_page("Behavior",		&prefs_page::behavior(preferences.clone()));
		prefs.borrow().add_page("Appearance",	&prefs_page::appearance(preferences.clone()));
		prefs.borrow().add_page("Plugins",		&prefs_page::plugins(preferences.clone()));

		if preferences.borrow().developer {
			prefs.borrow().add_page("Developer",	&prefs_page::developer(preferences.clone()));
		}


		window.show_all();

		prefs
	}
}
