use gtk::prelude::*;

use scout_core::Shared;
use crate::preferences::Preferences;

pub fn behavior(prefs: Shared<Preferences>) -> gtk::Box {
	let behavior = gtk::Box::new(gtk::Orientation::Vertical, 0);
	behavior.set_border_width(14);

	let category_label = gtk::Label::new(None);
	category_label.set_markup("<b>Fine tune Scout's behavior.</b>");
	category_label.set_widget_name("CategoryLabel");
	category_label.set_halign(gtk::Align::Start);
	behavior.pack_start(&category_label, false, false, 0);

	let unfocused_button = gtk::CheckButton::with_label("   Hide when the window is unfocused");
	unfocused_button.set_active(prefs.borrow().hide_on_unfocus);
	behavior.pack_start(&unfocused_button, false, false, 4);

	let preferences_clone = prefs.clone();
	unfocused_button.connect_toggled(move |s| preferences_clone.borrow_mut().hide_on_unfocus = s.get_active());

	let aot_button = gtk::CheckButton::with_label("   Keep window above other windows");
	aot_button.set_active(prefs.borrow().always_on_top);
	behavior.pack_start(&aot_button, false, false, 4);

	let preferences_clone = prefs.clone();
	aot_button.connect_toggled(move |s| preferences_clone.borrow_mut().always_on_top = s.get_active());

	behavior
}
