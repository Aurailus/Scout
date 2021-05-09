use gtk::prelude::*;

use core::SearchResult;

#[derive(Debug)]
pub struct FileResult {
	name: String,
	icon: String,
	path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct DirectoryResult {
	icon: String,
	path_str: String,
	description: String,
	path: std::path::PathBuf,

	widget: gtk::Box,
	top_button: gtk::Button
}

impl DirectoryResult {

	/**
	 *
	 */
	
	pub fn get_file_icon(path: &std::path::Path) -> String {
		let mime_type = mime_guess::from_path(&path).first()
			.and_then(|mime| Some(mime.to_string())).get_or_insert(String::new()).clone();

		if mime_type.is_empty() { return "folder".to_owned(); }
		
		let content_type = gio::content_type_from_mime_type(&mime_type.to_string()).unwrap().to_string();
		let icon = gio::content_type_get_icon(&content_type).unwrap().downcast::<gio::ThemedIcon>().unwrap()
			.get_names().first().and_then(|s| Some(s.to_string()));

		if icon.is_some() && !icon.as_ref().unwrap().is_empty() { icon.unwrap() } else { "text-x-script".to_owned() }
	}


	/**
	 *
	 */
	
	pub fn get_dir_icon(description: &str) -> &'static str {
		match description {
			"Home" => "user-home",
			"Documents" => "folder-documents",
			"Downloads" => "folder-download",
			"Music" => "folder-music",
			"Pictures" => "folder-pictures",
			"Public" => "folder-publicshare",
			"Templates" => "folder-templates",
			"Videos" => "folder-videos",
			_ => "folder",
		}
	}


	/**
	 *
	 */

	pub fn get_suggested_files(path: &std::path::Path) -> std::io::Result<Vec<FileResult>> {
		let mut files = std::fs::read_dir(path)?
			.map(|file| -> std::io::Result<(std::fs::DirEntry, std::fs::Metadata)> {
				let file = file?;
				let meta = file.metadata()?;
				Ok((file, meta))
			})
			.filter(|f| f.is_ok() && !f.as_ref().unwrap().0.file_name().to_str().unwrap().starts_with("."))
			.map(|f| f.unwrap())
			.collect::<Vec<_>>();

		files.sort_by(|(_, meta_a), (_, meta_b)| meta_b.accessed().unwrap().cmp(&meta_a.accessed().unwrap()));
		let files = files.into_iter()
			.map(|(file, _)| file)
			.take(5)
			.map(|file| FileResult {
				name: file.file_name().to_str().unwrap().to_owned(),
				icon: DirectoryResult::get_file_icon(file.file_name().as_ref()),
				path: file.path()
			})
			.collect::<Vec<_>>();

		Ok(files)
	}


	/**
	 * Creates a new Directory result, with a corresponding result widget.
	 */

	pub fn new(description: Option<&str>, path: &std::path::Path) -> Self {
		let description = description.unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap()).to_owned();
		let icon = DirectoryResult::get_dir_icon(&description).to_owned();
		
		let home_str = dirs::home_dir().and_then(|dir| Some(dir.to_str().unwrap().to_owned())).unwrap();
		let mut path_str = format!("{}/", path.to_str().unwrap().to_owned());
		if path_str.starts_with(&home_str) && path_str.len() > home_str.len() + 1 { path_str = path_str[home_str.len() + 1..].to_owned(); }
		
		let files = DirectoryResult::get_suggested_files(&path);

		let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
		widget.get_style_context().add_class("Application");
		widget.set_widget_name("SearchResult");
		
		let top_button = gtk::Button::new();
		top_button.get_style_context().add_class("flat");
		widget.pack_start(&top_button, true, true, 0);
		let path_clone = path.to_owned();
		top_button.connect_clicked(move |_| drop(opener::open(path_clone.to_str().unwrap())));

		{

			let widget_top = gtk::Box::new(gtk::Orientation::Horizontal, 4);
			top_button.add(&widget_top);

			let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
			icon_box.get_style_context().add_class("IconBox");
			widget_top.pack_start(&icon_box, false, false, 4);

			let icon = gtk::Image::from_icon_name(Some(&icon), gtk::IconSize::Dnd);
			icon.set_size_request(32, 32);
			icon_box.pack_start(&icon, false, false, 0);

			let description_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
			widget_top.pack_start(&description_box, true, true, 0);

			let category_label = gtk::Label::new(Some(&[ "<span size='small' weight='bold'>DIRECTORY</span>" ].join("")));
			category_label.get_style_context().add_class("Category");
			category_label.set_ellipsize(pango::EllipsizeMode::End);
			category_label.set_use_markup(true);
			category_label.set_xalign(0.0);
			description_box.pack_start(&category_label, false, false, 1);

			let label = gtk::Label::new(Some(&description));
			label.set_ellipsize(pango::EllipsizeMode::End);
			label.set_xalign(0.0);
			description_box.pack_start(&label, false, false, 1);

			if let Ok(files) = files {
				if files.len() > 0 {
					let widget_actions = gtk::Box::new(gtk::Orientation::Vertical, 0);
					widget.pack_start(&widget_actions, true, true, 0);

					for file in files {
						let widget_action_button = gtk::Button::new();
						widget_action_button.get_style_context().add_class("flat");
						widget_action_button.get_style_context().add_class("ActionButton");
						widget_actions.pack_start(&widget_action_button, true, true, 0);
						
						let path_clone = file.path.clone();
						widget_action_button.connect_clicked(move |_| drop(opener::open(path_clone.to_str().unwrap())));

						let widget_action = gtk::Box::new(gtk::Orientation::Horizontal, 0);
						widget_action_button.add(&widget_action);

						let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
						icon_box.get_style_context().add_class("IconBox");
						widget_action.pack_start(&icon_box, false, false, 4);

						let icon = gtk::Image::from_icon_name(Some(&file.icon), gtk::IconSize::Button);
						icon.set_size_request(16, 16);
						icon.set_pixel_size(16);
						icon_box.pack_start(&icon, false, false, 0);

						let action_label = gtk::Label::new(Some(&file.name));
						action_label.set_ellipsize(pango::EllipsizeMode::End);
						action_label.set_xalign(0.0);
						widget_action.pack_start(&action_label, false, false, 4);
					}
				}
			}
		}

		DirectoryResult {
			description, icon,
			path: path.to_owned(),
			path_str,
			top_button, widget
		}
	}
}

impl SearchResult for DirectoryResult {
	fn get_ranking(&self, query: &str) -> usize {
		let mut score = 0;
		let mut last_letter_ind: usize = 0;
		let mut lowercase_name = self.description.to_lowercase();
		lowercase_name.retain(|c| !c.is_whitespace());

		for letter in query.chars() {
			let mut name_chars = lowercase_name.chars().skip(last_letter_ind);
			let pos = name_chars.position(|c| c == letter).map_or(-1, |c| c as isize);
			if pos >= 0 {
				last_letter_ind += pos as usize + 1;
				score += std::cmp::max(10 - pos, 0) as usize
			}
		}

		score
	}

	fn set_first(&self, first: bool) -> () {
		self.top_button.set_can_focus(!first);
	}

	fn activate(&self) {
		drop(opener::open(self.path.to_str().unwrap()));
	}

	fn get_result_widget(&self) -> gtk::Widget {
		self.widget.clone().upcast()
	}

	fn get_preview_widget(&self) -> gtk::Widget {
		let widget = gtk::Box::new(gtk::Orientation::Vertical, 4);
		widget.get_style_context().add_class("Application");
		widget.set_widget_name("SearchPreview");
		widget.set_border_width(36);

		// let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
		// icon_box.get_style_context().add_class("IconBox");
		// icon_box.set_halign(gtk::Align::Center);
		// widget.pack_start(&icon_box, false, false, 0);

		// let icon = ApplicationResult::get_icon(self.icon.as_ref().and_then(|s| Some(s.as_str())), 96);
		// icon_box.pack_start(&icon, false, false, 0);

		// let category_label = gtk::Label::new(Some(&[ "<span size='small' weight='bold'>", &self.category, "</span>" ].join("")));
		// category_label.get_style_context().add_class("Category");
		// category_label.set_ellipsize(pango::EllipsizeMode::End);
		// category_label.set_use_markup(true);
		// widget.pack_start(&category_label, false, false, 0);

		// let label = gtk::Label::new(Some(&self.name));
		// label.set_ellipsize(pango::EllipsizeMode::End);
		// widget.pack_start(&label, false, false, 4);

		// let description = gtk::Label::new(Some(&[ &self.description, "." ].join("")));
		// description.get_style_context().add_class("Description");

		// description.set_line_wrap_mode(pango::WrapMode::WordChar);
		// description.set_ellipsize(pango::EllipsizeMode::End);
		// description.set_justify(gtk::Justification::Center);
		// description.set_halign(gtk::Align::Center);
		// description.set_max_width_chars(36);
		// description.set_line_wrap(true);
		// description.set_lines(5);

		// widget.pack_start(&description, false, false, 0);

		// let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		// button_box.get_style_context().add_class("ButtonBox");
		// button_box.get_style_context().add_class("linked");
		// button_box.set_halign(gtk::Align::Center);
		// button_box.set_valign(gtk::Align::End);
		// widget.pack_end(&button_box, false, false, 0);

		// let launch_button = gtk::Button::from_icon_name(Some("media-playback-start-symbolic"), gtk::IconSize::Button);
		// button_box.pack_start(&launch_button, false, false, 0);
		// let favorite_button = gtk::Button::from_icon_name(Some("emblem-favorite-symbolic"), gtk::IconSize::Button);
		// button_box.pack_start(&favorite_button, false, false, 0);
		// let edit_button = gtk::Button::from_icon_name(Some("document-edit-symbolic"), gtk::IconSize::Button);
		// button_box.pack_start(&edit_button, false, false, 0);

		return widget.upcast();
	}
}
