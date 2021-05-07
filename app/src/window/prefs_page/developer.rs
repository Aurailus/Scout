use gtk::prelude::*;

use crate::shared::Shared;
use crate::preferences::Preferences;

pub fn developer(_prefs: Shared<Preferences>) -> gtk::Box {
	let developer = gtk::Box::new(gtk::Orientation::Vertical, 0);
	developer.set_border_width(12);

	let category_label = gtk::Label::new(None);
	category_label.set_markup("<b>Developer options and references.</b>");
	category_label.set_widget_name("CategoryLabel");
	category_label.set_halign(gtk::Align::Start);
	developer.pack_start(&category_label, false, false, 2);

	let swatches = gtk::Box::new(gtk::Orientation::Vertical, 0);
	swatches.set_widget_name("SwatchTest");
	for i in 0..10 { swatches.pack_start(&gtk::Label::new(Some(&format!("@c-neutral-{}00", i))), false, false, 0); }
	developer.pack_start(&swatches, true, true, 0);

	developer
}
