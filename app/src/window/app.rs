use gio::prelude::*;
use gtk::prelude::*;
use scout_core::{ SearchResult };
use glib::translate::{ ToGlib, FromGlib };

use super::about;
use super::style;
use super::prefs::PrefsWindow;

use crate::shared::Shared;
use crate::plugins::Plugins;
use crate::preferences::Preferences;

static WIDTH: i32 = 700;
static HEIGHT: i32 = 500;

pub struct App {
	window: gtk::ApplicationWindow,
	search_entry: gtk::Entry,
	results_box: gtk::Box,
	preview_box: gtk::ScrolledWindow,

	plugins: Plugins,
	preferences: Shared<Preferences>,

	results: Vec<(usize, Box<dyn SearchResult>)>,
	top_result_focus_id: Option<glib::signal::SignalHandlerId>,

	pub last_hide: i64
}

impl App {
	pub fn new(application: &gtk::Application) -> Shared<Self> {
		let preferences = Preferences::new(None);

		let window = gtk::ApplicationWindow::new(application);
		window.set_icon_name(Some("system-search"));
		window.set_default_size(WIDTH, HEIGHT);
		window.set_decorated(false);
		window.set_resizable(false);
		window.set_title("Scout");
		style::style(&window, &preferences.borrow());
		window.get_style_context().add_class("Scout");
		
		window.set_skip_taskbar_hint(preferences.borrow().hide_on_unfocus);
		window.set_skip_pager_hint(preferences.borrow().hide_on_unfocus);
		window.set_keep_above(preferences.borrow().always_on_top);

		let app_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
		window.add(&app_container);

		let top_layout = gtk::Fixed::new();
		top_layout.set_widget_name("Header");
		app_container.pack_start(&top_layout, false, false, 0);

		let search = gtk::Entry::new();
		search.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("search-symbolic"));
		search.set_widget_name("SearchEntry");
		search.set_hexpand(true);
		search.set_size_request(WIDTH, 48);
		top_layout.put(&search, 0, 0);

		let profile = gtk::Button::new();
		profile.set_widget_name("ProfileButton");
		top_layout.put(&profile, WIDTH - 41, 7);

		let dropdown = gtk::PopoverMenu::new();
		dropdown.set_widget_name("ProfileDropdown");
		dropdown.set_relative_to(Some(&profile));
		dropdown.set_border_width(6);

		let dropdown_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
		dropdown.add(&dropdown_box);

		let greeting = gtk::ModelButton::new();
		greeting.set_property_text(Some(&[ "<span weight='bold'>", &whoami::realname(), "</span>" ].join("")));
		greeting.set_property_use_markup(true);
		greeting.set_action_name(Some("_"));
		dropdown_box.add(&greeting);
			
		dropdown_box.pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 3);

		let preferences_button = gtk::ModelButton::new();
		preferences_button.set_property_text(Some("Preferences"));
		preferences_button.set_action_name(Some("app.preferences"));
		dropdown_box.add(&preferences_button);

		let about_button = gtk::ModelButton::new();
		about_button.set_property_text(Some("About Scout"));
		about_button.set_action_name(Some("app.about"));
		dropdown_box.add(&about_button);

		dropdown_box.show_all();
		profile.connect_clicked(move |_| dropdown.popup());

		let profile_pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(
			&[ "/var/lib/AccountsService/icons/", &whoami::username() ].join(""), 32, 32, true).unwrap();
		let profile_image = gtk::Image::from_pixbuf(Some(&profile_pixbuf));
		profile.add(&profile_image);

		let content_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		content_container.set_widget_name("Content");
		app_container.pack_start(&content_container, true, true, 0);

		let results_scroller = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		results_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		results_scroller.set_size_request(220, -1);
		content_container.pack_start(&results_scroller, false, false, 0);

		let results = gtk::Box::new(gtk::Orientation::Vertical, 0);
		results.set_widget_name("ResultsFrame");
		results_scroller.add(&results);

		let preview = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		preview.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		preview.set_widget_name("PreviewFrame");
		content_container.pack_start(&preview, true, true, 0);

		if preferences.borrow().opacity != 100 { App::enable_transparency(&window); }
		window.show_all();
		
		let mut plugins = Plugins::new();
		plugins.load("/home/auri/Code/Projects/Scout/target/debug/libscout_plugin_application.so").expect("Invocation Failed");

		let app = Shared::new(App {
			window: window.clone(),
			search_entry: search.clone(),
			results_box: results.clone(),
			preview_box: preview.clone(),
			
			plugins,
			preferences: preferences.clone(),
			
			results: vec![],
			top_result_focus_id: None,

			last_hide: 0
		});

		let results_clone = results.clone();
		search.connect_property_has_focus_notify(move |search| {
			if search.has_focus() {
				results_clone.get_style_context().add_class("focus");
				let adj = results_scroller.get_vadjustment().unwrap();
				adj.set_value(adj.get_lower());
			}
			else { results_clone.get_style_context().remove_class("focus"); }
		});

		let app_clone = app.clone();
		search.connect_changed(move |_| if let Ok(mut app) = app_clone.try_borrow_mut() { app.search_changed(); });

		let app_clone = app.clone();
		search.connect_activate(move |_| {
			let app = app_clone.borrow();
			if app.results.len() > 0 { app.results[0].1.activate(); }
		});

		let app_clone = app.clone();
		search.connect_key_press_event(move |_, key| {
			if key.get_keyval() == gdk::keys::constants::Down {
				app_clone.borrow().results_box.child_focus(gtk::DirectionType::Down);
				gtk::Inhibit(true)
			}
			else { gtk::Inhibit(false) }
		});

		let search_clone = search.clone();
		results.connect_key_press_event(move |_, key| {
			let keyval = key.get_keyval();
			if keyval >= gdk::keys::constants::A && keyval <= gdk::keys::constants::z {
				search_clone.grab_focus();
				search_clone.emit_insert_at_cursor(&keyval.to_unicode().unwrap().to_string());
			}
			else if keyval == gdk::keys::constants::BackSpace {
				search_clone.emit_backspace();
				search_clone.grab_focus();
				search_clone.select_region(search_clone.get_text_length() as i32, search_clone.get_text_length() as i32)
			}
			gtk::Inhibit(false)
		});
		
		let actions = gio::SimpleActionGroup::new();
		window.insert_action_group("app", Some(&actions));

		let app_clone = app.clone();
		let preferences_action = gio::SimpleAction::new("preferences", None);
		preferences_action.connect_activate(move |_, _| {
			let mut app = app_clone.borrow_mut();
			PrefsWindow::new(&app.preferences.borrow());
			app.hide();
		});
		actions.add_action(&preferences_action);

		let about_action = gio::SimpleAction::new("about", None);
		about_action.connect_activate(|_, _| about::show_about());
		actions.add_action(&about_action);

		let first = Shared::new(true);
		let app_clone = app.clone();
		application.connect_activate(move |_| {
			if *first.borrow() {
				first.replace(false);
			}
			else {
				let mut app = app_clone.borrow_mut();
				if app.can_show() && !app.is_active() { app.show() }
				else { app.hide() }
			}
		});
		
		if preferences.borrow().hide_on_unfocus {
			let app_clone = app.clone();
			window.connect_focus_out_event(move |_, _| {
				app_clone.borrow_mut().hide();
				Inhibit(false)
			});
		}

		app
	}

	fn search_changed(&mut self) {
		let query = &self.search_entry.get_text().to_string().to_lowercase().replace(' ', "");

		// Clear current results
		if self.results.len() > 0 && self.top_result_focus_id.is_some() {
			self.results[0].1.get_result_widget().disconnect(
				glib::signal::SignalHandlerId::from_glib(self.top_result_focus_id.as_ref().unwrap().to_glib()));
			self.top_result_focus_id = None;
		}

		self.results_box.get_children().iter().for_each(|c| self.results_box.remove(c));
		self.preview_box.get_children().iter().for_each(|c| self.preview_box.remove(c));
		
		// Filter programs into search results.
		let mut results = self.plugins.get_results(&query);
		results.retain(|(score, _)| *score > 0);
		results.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
		let min = std::cmp::max(if results.len() >= 1 { (results[0].0 as f64 * 0.75) as usize } else { 0 }, query.len() * 5);
		results.retain(|(score, _)| *score >= min);
		self.results = results;

		if self.results.len() > 0 {
			self.preview_box.add(&self.results[0].1.get_preview_widget());

			for (i, res) in self.results.iter().enumerate() {
				res.1.set_first(i == 0);
				self.results_box.pack_start(&res.1.get_result_widget(), false, false, 0);
			}

			self.results_box.show_all();
			self.preview_box.show_all();
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

	fn hide(&mut self) {
		if !self.window.is_visible() { return }

		self.window.hide();
		self.last_hide = glib::get_monotonic_time();

		let search = self.search_entry.clone();
		drop(self);

		search.set_text("");
	}

	fn show(&mut self) {
		if !self.can_show() { return }

		self.window.grab_focus();
		self.window.show();
		self.search_entry.grab_focus();
	}
}
