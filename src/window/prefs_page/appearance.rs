use gtk::prelude::*;

use crate::shared::Shared;
use crate::preferences::Preferences;

pub fn appearance(prefs: Shared<Preferences>) -> gtk::Box {
	let appearance = gtk::Box::new(gtk::Orientation::Vertical, 0);
	appearance.set_border_width(12);

	let category_label = gtk::Label::new(None);
	category_label.set_markup("<b>Tweak the app's look and feel</b>");
	category_label.set_widget_name("CategoryLabel");
	category_label.set_halign(gtk::Align::Start);
	appearance.pack_start(&category_label, false, false, 2);

	let scale_label = gtk::Label::new(Some(&format!("Background Opacity  -  {}%", prefs.borrow().opacity)));
	scale_label.set_halign(gtk::Align::Start);
	appearance.pack_start(&scale_label, false, false, 4);

	let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 20.0, 100.0, 1.0);
	for i in 0..9 { scale.add_mark(i as f64 * 10.0 + 20.0, gtk::PositionType::Bottom, None); }
	scale.set_halign(gtk::Align::Start);
	scale.set_size_request(300, -1);
	scale.set_draw_value(false);
	scale.set_value(prefs.borrow().opacity as f64);

	let preferences_clone = prefs.clone();
	scale.connect_change_value(move |_, _, mut val| {
		val = val.min(100.0);
		scale_label.set_text(&format!("Background Opacity  -  {}%", val.floor()));
		preferences_clone.borrow_mut().opacity = val as u32;
		Inhibit(false)
	});

	appearance.pack_start(&scale, false, false, 0);

	appearance
}
