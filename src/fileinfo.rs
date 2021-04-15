use gio::prelude::Cast;
use std::convert::TryInto;
use gtk::prelude::IconThemeExt;

#[derive(PartialEq)]
pub enum FileType {
	File(String), // Mime Type
	Directory
}

pub struct FileInfo {
	pub name: String,
	pub path: std::path::PathBuf,
	pub size: usize,

	pub modified: std::time::SystemTime,

	pub is_symlink: bool,
	pub file_type: FileType,
	
	pub icon: String
}

impl FileInfo {
	pub fn from_dir_entry(dir_entry: std::fs::DirEntry) -> Self {
		FileInfo::new(&dir_entry.file_name().to_string_lossy(), &dir_entry.path(), &dir_entry.metadata().unwrap())
	}

	pub fn new(name: &str, path: &std::path::Path, metadata: &std::fs::Metadata) -> Self {
		let is_symlink = metadata.file_type().is_symlink();
		let metadata = if is_symlink { std::fs::metadata(&path).unwrap_or_else(move |_| metadata.clone()) } else { metadata.clone() };

		let file_type = if metadata.file_type().is_dir() { FileType::Directory } else { FileType::File(
			mime_guess::from_path(&path).first().and_then(|mime| Some(mime.to_string())).get_or_insert(String::new()).clone()) };

		let size: usize = match &file_type {
			FileType::File(_) => metadata.len().try_into().unwrap(),
			FileType::Directory => std::fs::read_dir(&path).and_then(|res| Ok(res.count())).unwrap_or(0)
		};

		let icon = match &file_type {
			FileType::File(mime_type) => FileInfo::icon_name_from_mime_type(&mime_type),
			FileType::Directory => FileInfo::icon_name_from_directory(path)
		};

		FileInfo {
			name: name.to_owned(),
			path: path.to_owned(),
			modified: metadata.modified().unwrap(),
			size, is_symlink,
			file_type, icon
		}
	}

	pub fn icon_name_from_directory(path: &std::path::Path) -> String {
		let path_str = path.to_string_lossy();
		let path_split: std::vec::Vec<&str> = path_str.split("/").filter(|seg| !seg.is_empty()).collect();
		if path_split.len() != 3 || path_split[0] != "home" { return "folder".to_owned(); }

		match path_split[2] {
			"Documents" => "folder-documents",
			"Downloads" => "folder-download",
			"Music" => "folder-music",
			"Pictures" => "folder-pictures",
			"Public" => "folder-publicshare",
			"Templates" => "folder-templates",
			"Videos" => "folder-videos",
			_ => "folder",
		}.to_owned()
	}

	pub fn icon_name_from_mime_type(mime_type: &str) -> String {
		let content_type = gio::content_type_from_mime_type(&mime_type.to_string()).unwrap().to_string();
		let icon_names = gio::content_type_get_icon(&content_type).unwrap().downcast::<gio::ThemedIcon>().unwrap().get_names();
 
		let mut icon = None;
		let theme = gtk::IconTheme::get_default();
		match theme {
			Some(theme) => {
				for name in &icon_names {
					let name_str = name.to_string();
					if theme.has_icon(&name_str) {
						icon = Some(name_str);
						break;
					}
				}
			},
			None => {
				if icon_names.len() > 0 { icon = Some(icon_names[0].to_string()); }
			}
		}

		icon.unwrap_or_else(|| "text-x-script".to_owned())
	}
}
