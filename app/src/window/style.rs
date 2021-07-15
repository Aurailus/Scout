#![allow(deprecated)]

use gtk::prelude::*;

use crate::preferences::Preferences;

pub fn style<T: IsA<gtk::Widget>>(window: &T, prefs: &Preferences, styles: &Vec<&'static str>) {
	let provider = gtk::CssProvider::new();

	let mut s = String::new();

	let mut add_color = |identifier: &str, color: &gdk::RGBA| {
		s.push_str("@define-color ");
		s.push_str(identifier);
		s.push_str(" ");
		s.push_str(&colorsys::Rgb::new(color.red * 255.0, color.green * 255.0, color.blue * 255.0, None).to_css_string());
		s.push_str(";\n");
	};

	let entry = gtk::Entry::new();
	let button = gtk::Button::new();

	add_color("c-neutral-000", &button.get_style_context().get_border_color(gtk::StateFlags::NORMAL));
	add_color("c-neutral-100", &window.get_style_context().get_background_color(gtk::StateFlags::NORMAL));
	add_color("c-neutral-900", &entry.get_style_context().get_color(gtk::StateFlags::NORMAL));

	s.push_str("@define-color c-background-primary alpha(@c-neutral-100, ");
	s.push_str(&(prefs.opacity as f64 / 100.0).to_string());
	s.push_str(");");

	s.push_str("@define-color c-background-secondary alpha(@c-neutral-200, ");
	s.push_str(&(prefs.opacity as f64 / 100.0).to_string());
	s.push_str(");");

	let style = include_str!("../../style/.build.css");
	s.push_str(style);
	for style in styles {
		s.push_str("\n");
		s.push_str(style);
	}

	provider.load_from_data(s.as_bytes()).expect("Failed to load CSS.");
	gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().expect("Error initializing GTK CSS provider."),
		&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}
