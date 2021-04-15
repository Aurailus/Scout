use gtk::prelude::*;

use super::style;
use crate::shared::Shared;
use crate::fileinfo::{ FileInfo, FileType };

pub struct App {
	deemphasized_color: String,

	location_history_head: usize,
	location_history: Vec<std::path::PathBuf>,

	location_view: gtk::Box,
	location_entry: gtk::Entry,
	navigate_back_button: gtk::Button,
	navigate_forward_button: gtk::Button,

	list_tree_store: gtk::TreeStore,
	completion_tree_store: gtk::ListStore
}

static SIZE_SUFFIXES: [ &str; 4 ] = [ "bytes", "KB", "MB", "GB" ];

impl App {
	fn add_column<T: IsA<gtk::CellRenderer>>(tree_view: &gtk::TreeView, cell: T, title: Option<&str>, attr: &str, ind: i32) {
		let column = gtk::TreeViewColumn::new();

		column.pack_start(&cell, false);
		if let Some(title) = title { column.set_title(title); }
		column.add_attribute(&cell, attr, ind);
		column.set_resizable(true);
		tree_view.append_column(&column);
	}

	pub fn new(app: &gtk::Application, location: &std::path::Path) -> Shared<Self> {
		let window = gtk::ApplicationWindow::new(app);
		window.set_title("File Explorer");
		window.set_icon_name(Some("system-file-manager"));

		let geom = gdk::Geometry {
			min_width: 900, min_height: 600,
			// Unused parameters, because for some reason gdk::Geometry doesn't provide Default >:(
			max_width: -1, max_height: -1, base_width: -1, base_height: -1,
			width_inc: -1, height_inc: -1, min_aspect: 0.0, max_aspect: 0.0,
			win_gravity: gdk::Gravity::Center
		};

		window.set_geometry_hints::<gtk::ApplicationWindow>(None, Some(&geom), gdk::WindowHints::MIN_SIZE);
		let deemphasized_color = style::style(&window);

		let header = gtk::HeaderBar::new();
		header.set_show_close_button(true);
		window.set_titlebar(Some(&header));

		let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		header_box.set_hexpand(true);
		header.set_custom_title(Some(&header_box));

		let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		button_box.get_style_context().add_class("linked");
		header_box.pack_start(&button_box, false, false, 3);

		let navigate_back_button = gtk::Button::from_icon_name(Some("go-previous-symbolic"), gtk::IconSize::Button);
		button_box.pack_start(&navigate_back_button, false, false, 0);
		
		let navigate_forward_button = gtk::Button::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::Button);
		button_box.pack_start(&navigate_forward_button, false, false, 0);

		// icon-name, name, path_str, path
		let completion_tree_store = gtk::ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type() ]);

		let completion_area = gtk::CellAreaBox::new();

		let completion_icon_renderer = gtk::CellRendererPixbuf::new();
		completion_icon_renderer.set_property_stock_size(gtk::IconSize::Menu);
		completion_icon_renderer.set_padding(6, 3);
		completion_area.add(&completion_icon_renderer);
		completion_area.add_attribute(&completion_icon_renderer, "icon-name", 0);

		let completion_name_renderer = gtk::CellRendererText::new();
		completion_area.add(&completion_name_renderer);
		completion_area.add_attribute(&completion_name_renderer, "text", 1);

		let completion_path_renderer = gtk::CellRendererText::new();
		completion_path_renderer.set_alignment(1.0, 0.0);
		completion_path_renderer.set_padding(6, 3);
		completion_area.add(&completion_path_renderer);
		completion_area.add_attribute(&completion_path_renderer, "markup", 2);

		let entry_completion = gtk::EntryCompletionBuilder::new().model(&completion_tree_store)
			.text_column(3).inline_completion(true).inline_selection(true).popup_single_match(false).cell_area(&completion_area).build();

		let location_entry = gtk::Entry::new();
		location_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("folder-symbolic"));
		location_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, Some("view-refresh-symbolic"));
		location_entry.set_icon_activatable(gtk::EntryIconPosition::Secondary, true);
		location_entry.set_completion(Some(&entry_completion));
		header_box.pack_start(&location_entry, true, true, 0);

		let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
		window.add(&paned);

		let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
		sidebar.set_size_request(200, 0);
		paned.pack1(&sidebar, false, false);

		let location_scroller = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
		location_scroller.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
		paned.pack2(&location_scroller, true, true);

		let location_view = gtk::Box::new(gtk::Orientation::Vertical, 0);
		location_view.set_hexpand(true);

		let tree_view = gtk::TreeView::new();
		location_scroller.add(&tree_view);
		
		// icon-name, name, type, size, modified, path
		let list_tree_store = gtk::TreeStore::new(&[ String::static_type(), String::static_type(),
			String::static_type(), String::static_type(), String::static_type(), String::static_type() ]);
		tree_view.set_model(Some(&list_tree_store));
		tree_view.get_selection().set_mode(gtk::SelectionMode::Multiple);

		let name_column = gtk::TreeViewColumn::new();
		name_column.set_resizable(true);
		name_column.set_expand(true);
		name_column.set_title(&"             Name");

		let name_icon_renderer = gtk::CellRendererPixbuf::new();
		name_icon_renderer.set_property_stock_size(gtk::IconSize::LargeToolbar);
		name_column.pack_start(&name_icon_renderer, false);

		let name_text_renderer = gtk::CellRendererText::new();
		name_column.pack_start(&name_text_renderer, true);

		name_column.add_attribute(&name_icon_renderer, "icon-name", 0);
		name_column.add_attribute(&name_text_renderer, "text", 1);
		tree_view.append_column(&name_column);

		App::add_column(&tree_view, gtk::CellRendererText::new(), Some(&"Type"),     "markup", 2);
		App::add_column(&tree_view, gtk::CellRendererText::new(), Some(&"Size"),     "markup", 3);
		App::add_column(&tree_view, gtk::CellRendererText::new(), Some(&"Modified"), "markup", 4);

		window.show_all();

		let app = Shared::new(App {
			deemphasized_color,

			location_history: vec![],
			location_history_head: 0,

			location_view,
			location_entry: location_entry.clone(),
			navigate_back_button: navigate_back_button.clone(),
			navigate_forward_button: navigate_forward_button.clone(),

			list_tree_store: list_tree_store.clone(),
			completion_tree_store: completion_tree_store.clone()
		});

		let app_clone = app.clone();
		location_entry.connect_activate(move |entry| drop(app_clone.borrow_mut().push_location(std::path::Path::new(&entry.get_text()))));

		let app_clone = app.clone();
		navigate_back_button.connect_clicked(move |_| drop(app_clone.borrow_mut().navigate_back()));

		let app_clone = app.clone();
		navigate_forward_button.connect_clicked(move |_| drop(app_clone.borrow_mut().navigate_forward()));

		let app_clone = app.clone();
		let list_tree_store_clone = list_tree_store.clone();
		tree_view.connect_row_activated(move |_, path, _| drop(app_clone.borrow_mut().path_activate(&std::path::Path::new(
			&list_tree_store_clone.get_value(&list_tree_store_clone.get_iter(path).unwrap(), 5).downcast::<String>().unwrap().get().unwrap()))));

		let app_clone = app.clone();
		let list_tree_store_clone = list_tree_store.clone();
		tree_view.connect_row_expanded(move |_, iter, path| {
			let path = std::path::Path::new(&list_tree_store_clone.get_value(&list_tree_store.get_iter(path).unwrap(), 5)
				.downcast::<String>().unwrap().get().unwrap()).to_owned();
			drop(app_clone.borrow_mut().populate_view(&path, Some(&iter)));
		});

		let app_clone = app.clone();
		window.connect_key_press_event(move |_, key| {
			let alt = key.get_state().intersects(gdk::ModifierType::MOD1_MASK);
			match key.get_keyval() {
				gdk::keys::constants::Left => {
					gtk::Inhibit(if !alt { false }
					else { drop(app_clone.borrow_mut().navigate_back()); true })
				},
				gdk::keys::constants::Right => {
					gtk::Inhibit(if !alt { false }
					else { drop(app_clone.borrow_mut().navigate_forward()); true })
				},
				gdk::keys::constants::Up => {
					gtk::Inhibit(if !alt { false }
					else { drop(app_clone.borrow_mut().navigate_up()); true })
				},
				_ => gtk::Inhibit(false)
			}
		});

		app.borrow_mut().push_location(location).unwrap();

		app
	}

	fn path_activate(&mut self, location: &std::path::Path) -> Result<(), &'static str> {
		let meta = std::fs::metadata(&location).or(Err("Path does not exist."))?;
		if meta.is_dir() { self.push_location(location) }
		else if open::that(location).is_err() { Err("Error opening the file.") }
		else { Ok(()) }
	}

	pub fn push_location(&mut self, location: &std::path::Path) -> Result<(), &'static str> {
		let location = location.canonicalize().or(Err("Path is invalid."))?;
		let meta = std::fs::metadata(&location).or(Err("Path does not exist."))?;
		if !meta.is_dir() { return Err("Path is not a directory."); }

		while self.location_history_head + 1 < self.location_history.len() { self.location_history.pop(); }
		self.location_history_head = self.location_history.len();
		self.location_history.push(location.to_owned());

		self.location_changed()
	}

	// pub fn has_up(&self) -> bool {
	// 	let location = &self.location_history[self.location_history_head];
	// 	location.parent().is_some()
	// }

	pub fn navigate_up(&mut self) -> Result<std::path::PathBuf, ()> {
		let location = self.location_history[self.location_history_head].clone();
		if let Some(up_location) = location.parent() {
			match self. push_location(&up_location) {
				Ok(_) => Ok(up_location.to_owned()),
				Err(_) => Err(())
			}
		}
		else { Err(()) }
	}

	pub fn has_back(&self) -> bool {
		self.location_history_head > 0
	}

	pub fn navigate_back(&mut self) -> Result<std::path::PathBuf, ()> {
		if !self.has_back() { return Err(()); }
		self.location_history_head -= 1;
		self.location_changed().unwrap();

		return Ok(self.location_history[self.location_history_head].clone());
	}

	pub fn has_forward(&self) -> bool {
		self.location_history_head + 1 < self.location_history.len()
	}

	pub fn navigate_forward(&mut self) -> Result<std::path::PathBuf, ()> {
		if !self.has_forward() { return Err(()); }
		self.location_history_head += 1;
		self.location_changed().unwrap();
		
		return Ok(self.location_history[self.location_history_head].clone());
	}

	fn get_sorted_dir_infos(&self, dir: std::fs::ReadDir) -> std::vec::Vec<FileInfo> {
		let mut entries: std::vec::Vec<FileInfo> =
			dir.filter(|dir| dir.is_ok()).map(|dir| FileInfo::from_dir_entry(dir.unwrap())).collect();

		entries.sort_unstable_by(|a, b| {
			if a.file_type == FileType::Directory && b.file_type != FileType::Directory { return std::cmp::Ordering::Less; }
			if a.file_type != FileType::Directory && b.file_type == FileType::Directory { return std::cmp::Ordering::Greater; }
			a.name.to_lowercase().cmp(&b.name.to_lowercase())
		});

		entries
	}

	fn format_size(file_type: &FileType, size: usize) -> String {
		match file_type {
			FileType::File(_) => {
				let mut suffix_ind = 0;
				let mut size: f32 = size as f32;
				while size > 1024.0 && suffix_ind < 4 { size /= 1024.0; suffix_ind += 1; }
				if suffix_ind == 0 { std::format!("{:.0} {}", size, SIZE_SUFFIXES[suffix_ind]) }
				else { std::format!("{:.1} {}", size, SIZE_SUFFIXES[suffix_ind]) }
			},
			FileType::Directory => std::format!("{} items", size),
		}
	}

	fn format_deemphasized(&self, string: &str) -> String {
		[ "<span foreground=\"", &self.deemphasized_color, "\">", string, "</span>" ].join("")
	}

	fn location_changed(&mut self) -> Result<(), &'static str> {
		self.navigate_back_button.set_sensitive(self.has_back());
		self.navigate_forward_button.set_sensitive(self.has_forward());

		let location = &self.location_history[self.location_history_head].clone();
		let mut location_string = location.to_string_lossy().to_string();
		location_string.push_str("/");

		self.populate_view(&location, None)?;

		self.completion_tree_store.clear();
		self.location_entry.set_text(&location_string);

		let hide: std::vec::Vec<String> = std::fs::read_to_string(location.join(std::path::Path::new(".hidden")))
			.and_then(|file| Ok(file.split("\n").map(|line| line.trim().to_owned()).filter(|line| !line.is_empty()).collect())).ok().unwrap_or(vec![]);

		for entry in self.get_sorted_dir_infos(std::fs::read_dir(&location).map_err(|_| "Failed to read directory.")?) {
			if entry.name.starts_with(".") || entry.name.ends_with("~") ||
				hide.contains(&entry.name) || entry.file_type != FileType::Directory { continue; }

			let path = entry.path.to_string_lossy().to_string();
			let path_styled = self.format_deemphasized(&path);

			let mut icon = entry.icon.clone();
			icon.push_str("-symbolic");
			self.completion_tree_store.insert_with_values(None, &[ 0, 1, 2, 3 ], &[ &icon, &entry.name, &path_styled, &path ]);
		};
			
		self.location_view.show_all();
		Ok(())
	}

	fn populate_view(&mut self, location: &std::path::Path, parent: Option<&gtk::TreeIter>) -> Result<(), &'static str> {
		let mut first = None;
		if let Some(iter) = self.list_tree_store.iter_children(parent) {
			first = Some(iter.clone());
			if self.list_tree_store.iter_next(&iter) {
				while self.list_tree_store.remove(&iter) { /* do-while */ }
			}
		}

		let hide: std::vec::Vec<String> = std::fs::read_to_string(location.join(std::path::Path::new(".hidden")))
			.and_then(|file| Ok(file.split("\n").map(|line| line.trim().to_owned()).filter(|line| !line.is_empty()).collect())).ok().unwrap_or(vec![]);

		for entry in self.get_sorted_dir_infos(location.read_dir().map_err(|_| "Failed to read directory.")?) {
			if entry.name.starts_with(".") || entry.name.ends_with("~") || hide.contains(&entry.name) { continue; }

			let time: chrono::DateTime<chrono::offset::Local> = entry.modified.into();

			let parent = self.list_tree_store.insert_with_values(parent, None, &[ 0, 1, 2, 3, 4, 5 ], &[
				&entry.icon, &entry.name,
				&self.format_deemphasized("Folder"),
				&self.format_deemphasized(&App::format_size(&entry.file_type, entry.size)),
				&self.format_deemphasized(&format!("{}", time.format("%b %e, %R"))),
				&entry.path.to_string_lossy().to_string()
			]);

			if entry.file_type == FileType::Directory && entry.size > 0 {
				self.list_tree_store.insert_with_values(Some(&parent), None, &[ 0, 1 ], &[ &"window-minimize-symbolic", &"..." ]); }
		};

		if let Some(iter) = first { self.list_tree_store.remove(&iter); }

		Ok(())
	}
}
