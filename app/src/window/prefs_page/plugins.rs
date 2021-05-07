use gtk::prelude::*;

use crate::shared::Shared;
use crate::preferences::Preferences;

pub fn plugins(_prefs: Shared<Preferences>) -> gtk::Box {
	let plugins = gtk::Box::new(gtk::Orientation::Vertical, 0);
	plugins.set_border_width(12);

	let category_label = gtk::Label::new(None);
	category_label.set_markup("<b>Find more with Plugins.</b>");
	category_label.set_widget_name("CategoryLabel");
	category_label.set_halign(gtk::Align::Start);
	plugins.pack_start(&category_label, false, false, 2);

	plugins
}
