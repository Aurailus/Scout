use gtk::prelude::*;

use crate::shared::Shared;
use crate::preferences::Preferences;

pub fn general(_prefs: Shared<Preferences>) -> gtk::Box {
	let general = gtk::Box::new(gtk::Orientation::Vertical, 0);
	general.set_border_width(12);

	let category_label = gtk::Label::new(None);
	category_label.set_markup("<b>General settings</b>");
	category_label.set_widget_name("CategoryLabel");
	category_label.set_halign(gtk::Align::Start);
	general.pack_start(&category_label, false, false, 2);

	general
}
