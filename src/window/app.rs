use gio::prelude::*;
use gtk::prelude::*;
use glib::translate::{ ToGlib, FromGlib };

use super::about;
use super::style;
use crate::shared::Shared;
use crate::result::{ SearchResult, ProgramResult };

static WIDTH: i32 = 700;
static HEIGHT: i32 = 500;

pub struct App {
	window: gtk::ApplicationWindow,
	search: gtk::Entry,
	results: gtk::Box,
	preview: gtk::ScrolledWindow,

	programs: Vec<ProgramResult>,
	top_result: Option<Box<dyn SearchResult>>,
	top_result_focus_id: Option<glib::signal::SignalHandlerId>
}

impl App {

	pub fn new(app: &gtk::Application) -> Shared<Self> {
		let window = gtk::ApplicationWindow::new(app);
		window.set_icon_name(Some("system-search"));
		window.set_default_size(WIDTH, HEIGHT);
		window.set_resizable(false);
		style::style(&window);
		window.get_style_context().add_class("Scout");
		window.set_app_paintable(true);
		window.set_decorated(false);
		window.set_title("Scout");

		window.connect_screen_changed(set_visual);
		window.connect_draw(draw);

		let app_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
		app_container.get_style_context().add_class("app_container");
		window.add(&app_container);

		let top_layout = gtk::Fixed::new();
		top_layout.set_widget_name("top_layout");
		app_container.pack_start(&top_layout, false, false, 0);

		let search = gtk::Entry::new();
		search.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("search-symbolic"));
		search.set_widget_name("search");
		search.set_hexpand(true);
		search.set_size_request(WIDTH, 48);
		top_layout.put(&search, 0, 0);

		let profile = gtk::Button::new();
		profile.set_widget_name("profile");
		top_layout.put(&profile, WIDTH - 41, 7);

		let dropdown = gtk::PopoverMenu::new();
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

		let preferences = gtk::ModelButton::new();
		preferences.set_property_text(Some("Preferences"));
		preferences.set_action_name(Some("app.preferences"));
		dropdown_box.add(&preferences);

		let about = gtk::ModelButton::new();
		about.set_property_text(Some("About Scout"));
		about.set_action_name(Some("app.about"));
		dropdown_box.add(&about);

		dropdown_box.show_all();
		profile.connect_clicked(move |_| dropdown.popup());

		let profile_pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale("/var/lib/AccountsService/icons/auri", 32, 32, true).unwrap();
		let profile_image = gtk::Image::from_pixbuf(Some(&profile_pixbuf));
		profile.add(&profile_image);

		let content_container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		app_container.pack_start(&content_container, true, true, 0);

		let results_scroller = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		results_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		results_scroller.set_size_request(220, -1);
		content_container.pack_start(&results_scroller, false, false, 0);

		let results = gtk::Box::new(gtk::Orientation::Vertical, 0);
		results.set_widget_name("results");
		results_scroller.add(&results);

		let preview = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		preview.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		preview.set_widget_name("preview");
		content_container.pack_start(&preview, true, true, 0);


		set_visual(&window, None);
		window.show_all();

		let app = Shared::new(App {
			window: window.clone(),
			search: search.clone(),
			results: results.clone(),
			preview: preview.clone(),
			
			programs: ProgramResult::find_all(),
			
			top_result: None,
			top_result_focus_id: None
		});

		let results_clone = results.clone();
		search.connect_property_has_focus_notify(move |search| {
			if search.has_focus() {
				results_clone.get_style_context().add_class("pseudo_focus");
				let adj = results_scroller.get_vadjustment().unwrap();
				adj.set_value(adj.get_lower());
			}
			else { results_clone.get_style_context().remove_class("pseudo_focus"); }
		});

		let app_clone = app.clone();
		search.connect_changed(move |s| app_clone.borrow_mut().search_changed(
			&s.get_text().to_string().to_lowercase().replace(' ', "")));

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

		let about = gio::SimpleAction::new("about", None);
		about.connect_activate(|_, _| about::show_about());
		actions.add_action(&about);

		app
	}

	fn search_changed(&mut self, query: &str) {
		
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

		self.window.show_all();
	}
}

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
