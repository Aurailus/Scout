use gio::prelude::*;
use gtk::prelude::*;
use scout_core::{ SearchResult };
// use glib::translate::{ ToGlib, FromGlib };

use super::about;
use super::style;
use super::prefs::PrefsWindow;

use scout_core::Shared;
// use crate::plugins::Plugins;
use crate::preferences::Preferences;

static DIMENSIONS: (i32, i32) = ( 700, 500 );

pub struct WindowCallbacks {
	pub on_input: Box<dyn FnMut(&str)>,
	pub on_submit: Box<dyn FnMut()>,
	// on_clear: Box<dyn FnMut()>,
	// on_show: Box<dyn FnMut()>,
	// on_hide: Box<dyn FnMut()>
}

impl Default for WindowCallbacks {
	fn default() -> Self {
		WindowCallbacks {
			on_input: Box::new(|_| ()),
			on_submit: Box::new(|| ()),
		}
	}
}

pub struct Window {
	window: gtk::ApplicationWindow,
	search_entry: gtk::Entry,
	results_box: gtk::Box,
	results_scroller: gtk::ScrolledWindow,
	preview_scroller: gtk::ScrolledWindow,

	callbacks: Shared<WindowCallbacks>,
	preferences: Shared<Preferences>,

	results: Vec<Box<dyn SearchResult>>,

	pub last_hide: i64
}

impl Window {
	pub fn new(gtk: &gtk::Application, styles: &Vec<&'static str>) -> Shared<Self> {
		let preferences = Preferences::new(None);
		let window = gtk::ApplicationWindow::new(gtk);

		// Basic window configuration //

		window.set_icon_name(Some("system-search"));
		window.set_default_size(DIMENSIONS.0, DIMENSIONS.1);
		window.set_decorated(false);
		window.set_resizable(false);
		window.set_title("Scout");
		window.get_style_context().add_class("Scout");

		window.set_skip_taskbar_hint(preferences.borrow().hide_on_unfocus);
		window.set_skip_pager_hint(preferences.borrow().hide_on_unfocus);
		window.set_keep_above(preferences.borrow().always_on_top);

		style::style(&window, &preferences.borrow(), styles);

		let app_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
		window.add(&app_container);

		// Header, search, and profile button //

		let top_layout = gtk::Fixed::new();
		top_layout.set_widget_name("Header");
		app_container.pack_start(&top_layout, false, false, 0);

		let search_entry = gtk::Entry::new();
		search_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("search-symbolic"));
		search_entry.set_widget_name("SearchEntry");
		search_entry.set_hexpand(true);
		search_entry.set_size_request(DIMENSIONS.0, 48);
		top_layout.put(&search_entry, 0, 0);

		let profile = gtk::Button::new();
		profile.set_widget_name("ProfileButton");
		top_layout.put(&profile, DIMENSIONS.0 - 41, 7);

		let profile_pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(
			&[ "/var/lib/AccountsService/icons/", &whoami::username() ].join(""), 32, 32, true);
		if let Ok(profile_pixbuf) = profile_pixbuf {
			let profile_image = gtk::Image::from_pixbuf(Some(&profile_pixbuf));
			profile.add(&profile_image);
		}

		// Profile dropdown and list items //

		let dropdown = gtk::PopoverMenu::new();
		dropdown.set_widget_name("ProfileDropdown");
		dropdown.set_relative_to(Some(&profile));
		dropdown.set_border_width(6);

		let dropdown_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
		dropdown.add(&dropdown_box);

		fn add_dropdown_button(dropdown: &gtk::Box, label: &str, icon: &str, action: &str) {
			let button = gtk::Button::new();
			button.get_style_context().add_class("flat");
			button.get_style_context().add_class("model");
			button.set_action_name(Some(&action));

			let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
			button.add(&button_box);

			let icon = gtk::Image::from_icon_name(Some(&icon), gtk::IconSize::Button);
			button_box.add(&icon);

			let label = gtk::Label::new(Some(&label));
			label.set_halign(gtk::Align::Start);
			button_box.add(&label);

			dropdown.add(&button);
		}

		add_dropdown_button(&dropdown_box, "Log Out", 	"system-log-out-symbolic", 	"system.logout");
		add_dropdown_button(&dropdown_box, "Shut Down", "system-shutdown-symbolic", "system.shutdown");
		add_dropdown_button(&dropdown_box, "Restart", 	"system-reboot-symbolic", 	"system.restart");

		dropdown_box.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 3);

		add_dropdown_button(&dropdown_box, "Preferences", 	"preferences-system-symbolic", 	"app.preferences");
		add_dropdown_button(&dropdown_box, "About Scout", 	"dialog-information-symbolic", 	"app.about");

		dropdown_box.show_all();
		profile.connect_clicked(move |_| dropdown.popup());

		// Result and preview containers //

		let content_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		content_container.set_widget_name("Content");
		app_container.pack_start(&content_container, true, true, 0);

		let results_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
		results_container.set_widget_name("ResultsContainer");
		content_container.pack_start(&results_container, false, false, 0);

		let results_scroller = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		results_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		results_scroller.set_widget_name("ResultsScroller");
		results_scroller.set_size_request(220, -1);
		results_container.pack_start(&results_scroller, true, true, 0);

		let results_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
		results_scroller.add(&results_box);

		let preview = gtk::Box::new(gtk::Orientation::Vertical, 0);
		preview.set_widget_name("PreviewContainer");
		content_container.pack_start(&preview, true, true, 0);

		let preview_scroller = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		preview_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		preview.pack_start(&preview_scroller, true, true, 0);

		// Final configuration //

		if preferences.borrow().opacity != 100 { Window::enable_transparency(&window); }
		window.show_all();

		// Create the window shared object

		let window = Shared::new(Window {
			window,
			search_entry,
			results_box,
			results_scroller,
			preview_scroller,

			callbacks: Shared::new(WindowCallbacks::default()),
			preferences,
			results: vec![],

			last_hide: 0
		});

		// Add focus to the first entry when the search entry is selected

		let search_entry_clone = window.borrow().search_entry.clone();
		let results_scroller_clone = window.borrow().results_scroller.clone();
		search_entry_clone.connect_property_has_focus_notify(move |search_entry| {
			if search_entry.has_focus() {
				results_scroller_clone.get_style_context().add_class("focus");
				let adj = results_scroller_clone.get_vadjustment().unwrap();
				adj.set_value(adj.get_lower());
			}
			else { results_scroller_clone.get_style_context().remove_class("focus"); }
		});

		// Bind search entry functions

		let callbacks_clone = window.borrow().callbacks.clone();
		window.borrow().search_entry.connect_changed(move |entry|
			(callbacks_clone.borrow_mut().on_input)(&entry.get_text().to_string()));

		let callbacks_clone = window.borrow().callbacks.clone();
		window.borrow().search_entry.connect_activate(move |_|
			(callbacks_clone.borrow_mut().on_submit)());

		// let window_clone = window.clone();

		// let search_clone = widgets.search.clone();
		// widgets.results.connect_key_press_event(move |_, key| {
		// 	let keyval = key.get_keyval();
		// 	if keyval >= gdk::keys::constants::A && keyval <= gdk::keys::constants::z {
		// 		search_clone.grab_focus();
		// 		search_clone.emit_insert_at_cursor(&keyval.to_unicode().unwrap().to_string());
		// 	}
		// 	else if keyval == gdk::keys::constants::BackSpace {
		// 		search_clone.emit_backspace();
		// 		search_clone.grab_focus();
		// 		search_clone.select_region(search_clone.get_text_length() as i32, search_clone.get_text_length() as i32)
		// 	}
		// 	gtk::Inhibit(false)
		// });

		// let app_clone = app.clone();
		// widgets.search.connect_activate(move |_| {
		// 	let app = app_clone.borrow();
		// 	if app.results.len() > 0 { app.results[0].1.activate(); }
		// });

		// Override key presses to change the active entry

		// let app_clone = app.clone();
		// widgets.search.connect_key_press_event(move |_, key| {
		// 	if key.get_keyval() == gdk::keys::constants::Down {
		// 		app_clone.borrow().widgets.as_ref().unwrap().results.child_focus(gtk::DirectionType::Down);
		// 		gtk::Inhibit(true)
		// 	}
		// 	else { gtk::Inhibit(false) }
		// });

		// Fake input manipulation when not focusing the search

		// Add dropdown functions

		let actions = gio::SimpleActionGroup::new();
		window.borrow().window.insert_action_group("app", Some(&actions));

		let window_clone = window.clone();
		let preferences_action = gio::SimpleAction::new("preferences", None);
		preferences_action.connect_activate(move |_, _| {
			let mut window = window_clone.borrow_mut();
			PrefsWindow::new(&window.preferences.borrow());
			window.hide();
		});
		actions.add_action(&preferences_action);

		let about_action = gio::SimpleAction::new("about", None);
		about_action.connect_activate(|_, _| about::show_about());
		actions.add_action(&about_action);

		let actions = gio::SimpleActionGroup::new();
		window.borrow().window.insert_action_group("system", Some(&actions));

		let logout_action = gio::SimpleAction::new("logout", None);
		logout_action.connect_activate(move |_, _| drop(std::process::Command::new("xfce4-session-logout").spawn()));
		actions.add_action(&logout_action);

		let shutdown_action = gio::SimpleAction::new("shutdown", None);
		shutdown_action.connect_activate(move |_, _| drop(system_shutdown::shutdown()));
		actions.add_action(&shutdown_action);

		let restart_action = gio::SimpleAction::new("restart", None);
		restart_action.connect_activate(move |_, _| drop(system_shutdown::reboot()));
		actions.add_action(&restart_action);

		// Update hidden state when the app is activated (move this to App)

		let first = Shared::new(true);
		let window_clone = window.clone();
		gtk.connect_activate(move |_| {
			if *first.borrow() {
				first.replace(false);
			}
			else {
				let mut window = window_clone.borrow_mut();
				if window.can_show() && !window.is_active() { window.show() }
				else { window.hide() }
			}
		});

		if window.borrow().preferences.borrow().hide_on_unfocus {
			let window_clone = window.clone();
			window.borrow().window.connect_focus_out_event(move |_, _| {
				window_clone.borrow_mut().hide();
				Inhibit(false)
			});
		}

		window
	}

	pub fn bind(&mut self, callbacks: WindowCallbacks) {
		self.callbacks.replace(callbacks);
	}

	pub fn set_results(&mut self, results: Vec<Box<dyn SearchResult>>) {
		self.results = results;

		self.results_box.get_children().iter()
			.for_each(|c| self.results_box.remove(c));
		self.preview_scroller.get_children().iter()
			.for_each(|c| self.preview_scroller.remove(c));

		if self.results.len() > 0 {
			self.preview_scroller.add(&self.results[0].get_preview_widget());

			for (i, res) in self.results.iter().enumerate() {
				res.set_first(i == 0);
				self.results_box.pack_start(&res.get_result_widget(), false, false, 0);
			}

			self.results_box.show_all();
			self.preview_scroller.show_all();
		}
	}

	fn enable_transparency(window: &gtk::ApplicationWindow) {
		fn set_visual(window: &gtk::ApplicationWindow, _: Option<&gdk::Screen>) {
			let screen = window.get_screen().unwrap();
			if let Some(ref visual) = screen.get_rgba_visual() { window.set_visual(Some(visual)); }
			else { println!("RGBA missing."); }
		}

		fn draw(_: &gtk::ApplicationWindow, ctx: &cairo::Context) -> Inhibit {
			ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
			ctx.set_operator(cairo::Operator::Screen);
			ctx.paint();
			Inhibit(false)
		}

		window.set_app_paintable(true);
		window.connect_draw(draw);
		window.connect_screen_changed(set_visual);
		set_visual(&window, None);
	}

	fn can_show(&self) -> bool {
		glib::get_monotonic_time() - self.last_hide >= 250_000
	}

	fn is_active(&self) -> bool {
		self.window.get_focus().is_some() && self.window.is_visible()
	}

	fn show(&mut self) {
		if !self.can_show() { return }

		self.window.show();
		self.search_entry.grab_focus();
	}

	fn hide(&mut self) {
		if !self.window.is_visible() { return }

		self.window.hide();
		self.last_hide = glib::get_monotonic_time();

		let search = self.search_entry.clone();
		drop(self);

		search.set_text("");
	}
}
