use gtk::prelude::*;

use super::style;
use crate::shared::Shared;
use crate::fileinfo::{ FileInfo, FileType };

pub struct App {
	location_history_head: usize,
	location_history: Vec<std::path::PathBuf>,

	location_view: gtk::Box,
	tree_store: gtk::TreeStore,

	location_entry: gtk::Entry,
	navigate_back_button: gtk::Button,
	navigate_forward_button: gtk::Button
}

impl App {
	fn add_column<T: IsA<gtk::CellRenderer>>(tree_view: &gtk::TreeView, cell: T, expand: bool, resize: bool, title: Option<&str>, attr: &str, ind: i32) {
		let column = gtk::TreeViewColumn::new();

		column.pack_start(&cell, expand);
		column.set_expand(expand);
		if let Some(title) = title { column.set_title(title); }
		column.add_attribute(&cell, attr, ind);
		column.set_resizable(resize);
		column.set_reorderable(resize);
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
		style::style(&window);

		let header = gtk::HeaderBar::new();
		header.set_show_close_button(true);
		header.set_decoration_layout(Some("icon:minimize,maximize,close"));
		window.set_titlebar(Some(&header));

		let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		header_box.set_hexpand(true);
		header.set_custom_title(Some(&header_box));

		let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
		button_box.get_style_context().add_class("linked");
		header_box.pack_start(&button_box, false, false, 4);

		let navigate_back_button = gtk::Button::from_icon_name(Some("go-previous-symbolic"), gtk::IconSize::Button);
		button_box.pack_start(&navigate_back_button, false, false, 0);
		
		let navigate_forward_button = gtk::Button::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::Button);
		button_box.pack_start(&navigate_forward_button, false, false, 0);

		let location_entry = gtk::Entry::new();
		location_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("folder-symbolic"));
		location_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Secondary, Some("view-refresh-symbolic"));
		location_entry.set_icon_activatable(gtk::EntryIconPosition::Secondary, true);
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
		location_scroller.add(&location_view);

		let tree_view = gtk::TreeView::new();
		location_view.pack_start(&tree_view, true, true, 0);
		
		// icon-name, name, type, size, modified, path
		let tree_store = gtk::TreeStore::new(&[ String::static_type(), String::static_type(),
			String::static_type(), String::static_type(), String::static_type(), String::static_type() ]);
		tree_view.set_model(Some(&tree_store));

		let icon_view = gtk::CellRendererPixbuf::new();
		icon_view.set_property_stock_size(gtk::IconSize::LargeToolbar);
		App::add_column(&tree_view, icon_view, 										false, false, None,              "icon-name", 0);
		App::add_column(&tree_view, gtk::CellRendererText::new(), true,  true,  Some(&"Name"),     "text",      1);
		App::add_column(&tree_view, gtk::CellRendererText::new(), false, true,  Some(&"Type"),     "text",      2);
		App::add_column(&tree_view, gtk::CellRendererText::new(), false, true,  Some(&"Size"),     "text",      3);
		App::add_column(&tree_view, gtk::CellRendererText::new(), false, true,  Some(&"Modified"), "text",      4);

		window.show_all();

		let app = Shared::new(App {
			location_history: vec![],
			location_history_head: 0,

			location_view,
			tree_store: tree_store.clone(),
			location_entry: location_entry.clone(),
			navigate_back_button: navigate_back_button.clone(),
			navigate_forward_button: navigate_forward_button.clone()
		});

		{
			let app_clone = app.clone();
			location_entry.connect_activate(move |entry| app_clone.borrow_mut().push_location(std::path::Path::new(&entry.get_text())));

			let app_clone = app.clone();
			navigate_back_button.connect_clicked(move |_| drop(app_clone.borrow_mut().navigate_back()));

			let app_clone = app.clone();
			navigate_forward_button.connect_clicked(move |_| drop(app_clone.borrow_mut().navigate_forward()));
		}

		{
			let app_clone = app.clone();
			let tree_store = tree_store.clone();
			tree_view.connect_row_activated(move |_, path, _| app_clone.borrow_mut().push_location(&std::path::Path::new(
				&tree_store.get_value(&tree_store.get_iter(path).unwrap(), 5).downcast::<String>().unwrap().get().unwrap())));
		}

		app.borrow_mut().push_location(location);

		app
	}

	pub fn push_location(&mut self, location: &std::path::Path) {
		match self.try_push_location(location) {
			Ok(_) => self.location_changed().unwrap(),
			Err(_) => ()
		}
	}

	fn try_push_location(&mut self, location: &std::path::Path) -> Result<(), &'static str> {
		let location = location.canonicalize().or(Err("Path is invalid."))?;
		let meta = std::fs::metadata(&location).or(Err("Path does not exist."))?;
		if !meta.is_dir() { return Err("Path is not a directory."); }

		while self.location_history_head + 1 < self.location_history.len() { self.location_history.pop(); }
		self.location_history_head = self.location_history.len();
		self.location_history.push(location.to_owned());

		Ok(())
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

	fn location_changed(&mut self) -> std::io::Result<()> {
		self.navigate_back_button.set_sensitive(self.has_back());
		self.navigate_forward_button.set_sensitive(self.has_forward());

		let location = &self.location_history[self.location_history_head];
		self.location_entry.set_text(&location.to_string_lossy());

		let hide: std::vec::Vec<String> = std::fs::read_to_string(location.join(std::path::Path::new(".hidden")))
			.and_then(|file| Ok(file.split("\n").map(|line| line.trim().to_owned()).filter(|line| !line.is_empty()).collect())).ok().unwrap_or(vec![]);

		self.tree_store.clear();

		for entry in self.get_sorted_dir_infos(std::fs::read_dir(&location)?) {
			if entry.name.starts_with(".") || entry.name.ends_with("~") || hide.contains(&entry.name) { continue; }

			self.tree_store.insert_with_values(None, None, &[ 0, 1, 2, 3, 4, 5 ],
				&[ &entry.icon, &entry.name, &"Folder", &entry.size.to_string(), &"Yesterday", &entry.path.to_string_lossy().to_string() ]);
		};
			
		self.location_view.show_all();
		Ok(())
	}
}
