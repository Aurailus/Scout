use gio::prelude::*;
use gtk::prelude::*;
use glib::translate::{ ToGlib, FromGlib };

use super::about;
use super::style;
use super::prefs::PrefsWindow;

use crate::shared::Shared;
use crate::preferences::Preferences;
use crate::result::{ SearchResult, ProgramResult };

static WIDTH: i32 = 700;
static HEIGHT: i32 = 500;

pub struct App {
	window: gtk::ApplicationWindow,
	search: gtk::Entry,
	results: gtk::Box,
	preview: gtk::ScrolledWindow,

	preferences: Shared<Preferences>,

	programs: Vec<ProgramResult>,
	top_result: Option<Box<dyn SearchResult>>,
	top_result_focus_id: Option<glib::signal::SignalHandlerId>
}

impl App {
	pub fn new(application: &gtk::Application) -> Shared<Self> {
		let preferences = Preferences::new(None);

		let window = gtk::ApplicationWindow::new(application);
		window.set_icon_name(Some("system-search"));
		window.set_default_size(WIDTH, HEIGHT);
		window.set_skip_taskbar_hint(true);
		window.set_skip_pager_hint(true);
		window.set_decorated(false);
		window.set_resizable(false);
		window.set_title("Scout");
		style::style(&window, &preferences.borrow());
		window.get_style_context().add_class("Translucent");
		window.get_style_context().add_class("Scout");

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

		let app = Shared::new(App {
			window: window.clone(),
			search: search.clone(),
			results: results.clone(),
			preview: preview.clone(),
			
			preferences,
			programs: ProgramResult::find_all(),
			
			top_result: None,
			top_result_focus_id: None
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
			if let Some(res) = app.top_result.as_ref() { res.activate(); }
		});

		let app_clone = app.clone();
		search.connect_key_press_event(move |_, key| {
			if key.get_keyval() == gdk::keys::constants::Down {
				app_clone.borrow().results.child_focus(gtk::DirectionType::Down);
				gtk::Inhibit(true)
			}
			else { gtk::Inhibit(false) }
		});

		let app_clone = app.clone();
		results.connect_key_press_event(move |_, key| {
			let keyval = key.get_keyval();
			let search = app_clone.borrow().search.clone();
			
			if keyval >= gdk::keys::constants::A && keyval <= gdk::keys::constants::z {
				search.grab_focus();
				search.emit_insert_at_cursor(&keyval.to_unicode().unwrap().to_string());
			}
			else if keyval == gdk::keys::constants::BackSpace {
				search.emit_backspace();
				search.grab_focus();
				search.select_region(search.get_text_length() as i32, search.get_text_length() as i32)
			}
			gtk::Inhibit(false)
		});
		 
		let actions = gio::SimpleActionGroup::new();
		window.insert_action_group("app", Some(&actions));

		let app_clone = app.clone();
		let preferences = gio::SimpleAction::new("preferences", None);
		preferences.connect_activate(move |_, _| { PrefsWindow::new(&app_clone.borrow().preferences.borrow()); });

		actions.add_action(&preferences);

		let about = gio::SimpleAction::new("about", None);
		about.connect_activate(|_, _| about::show_about());
		actions.add_action(&about);

		let last_unfocus = Shared::new(0);

		let app_clone = app.clone();
		let last_unfocus_clone = last_unfocus.clone();
		application.connect_activate(move |_| {
			let mut app = app_clone.borrow_mut();
			let last_unfocus = last_unfocus_clone.borrow().to_owned();
			if !app.window.is_visible() && glib::get_monotonic_time() - last_unfocus > 250_000 {
				app.window.show();
				app.search.grab_focus();
				app.search.select_region(search.get_text_length() as i32, search.get_text_length() as i32);
			}
			else if last_unfocus != 0 {
				app.window.hide();
				app.search.set_text("");
				app.search_changed();
			}
		});
		
		let app_clone = app.clone();
		let last_unfocus_clone = last_unfocus.clone();
		window.connect_focus_out_event(move |window, _| {
			last_unfocus_clone.replace(glib::get_monotonic_time());
			window.hide();
			let mut app = app_clone.borrow_mut();
			app.search.set_text("");
			app.search_changed();
			Inhibit(false)
		});

		app
	}

	fn search_changed(&mut self) {
		let query = &self.search.get_text().to_string().to_lowercase().replace(' ', "");

		// Clear current results
		if self.top_result.is_some() && self.top_result_focus_id.is_some() {
			self.top_result.as_ref().unwrap().get_result_widget().disconnect(
				glib::signal::SignalHandlerId::from_glib(self.top_result_focus_id.as_ref().unwrap().to_glib()));
			self.top_result_focus_id = None;
		}

		self.results.get_children().iter().for_each(|c| self.results.remove(c));
		self.preview.get_children().iter().for_each(|c| self.preview.remove(c));
		
		// Filter programs into search results.
		let mut results = self.programs.clone().into_iter().map(|app| (app.get_ranking(&query), app))
			.filter(|(score, _)| *score > 0).collect::<Vec<_>>();
		results.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap());
		let min = std::cmp::max(if results.len() >= 1 { (results[0].0 as f64 * 0.75) as usize } else { 0 }, query.len() * 5);
		results = results.into_iter().filter(|(score, _)| *score >= min).collect::<Vec<_>>();

		// Populate new results.
		self.top_result = match results.len() {
			0 => None,
			_ => {
				self.preview.add(&results[0].1.get_preview_widget());

				for (i, res) in results.iter().enumerate() {
					res.1.set_first(i == 0);
					self.results.pack_start(&res.1.get_result_widget(), false, false, 0);
				}

				Some(Box::new(results[0].1.clone()))
			}
		};

		self.results.show_all();
		self.preview.show_all();
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
}
