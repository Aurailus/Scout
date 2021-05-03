use gtk::prelude::*;

pub fn show_about() {
	let about = gtk::AboutDialogBuilder::new().use_header_bar(1).build();
	
	about.set_program_name("Scout");
	about.set_version(Some("0.0.1"));
	about.set_license_type(gtk::License::Gpl30);
	about.set_logo_icon_name(Some("system-search"));
	about.set_icon_name(Some("dialog-information"));
	about.set_copyright(Some("© 2021 Auri Collings"));
	about.set_website(Some("https://scout.aurailus.com"));
	about.set_comments(Some("System-wide search for GTK."));
	about.add_credit_section("Created by", &[ "Auri Collings" ]);

	let titlebar = about.get_titlebar().unwrap().downcast::<gtk::HeaderBar>().unwrap();
	for child in titlebar.get_children() {
		if let Ok(button) = child.downcast::<gtk::Button>() { titlebar.remove(&button); }
	}

	about.connect_response(|about, _| about.close());
	about.run();
}
