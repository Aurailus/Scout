use core::{ Plugin, SearchResult, PluginRegistrar };

use std::path::PathBuf;

mod result;
use result::{ DirectoryResult };

pub struct DirectoryPlugin {
	results: Vec<DirectoryResult>
}

impl DirectoryPlugin {
	fn add_directory(&mut self, description: Option<&str>, path: Option<PathBuf>) {
		if let Some(path) = path {
			let result = DirectoryResult::new(description, &path);
			self.results.push(result);
			// println!("{:?}", path);
		}
	}

	fn new() -> Box<dyn Plugin> {
		gtk::init().unwrap();
		
		let mut plugin = Box::new(DirectoryPlugin {
			results: vec![]
		});

		plugin.add_directory(None, dirs::audio_dir());
		plugin.add_directory(None, dirs::desktop_dir());
		plugin.add_directory(None, dirs::document_dir());
		plugin.add_directory(None, dirs::download_dir());
		plugin.add_directory(Some("Applications"), dirs::executable_dir());
		plugin.add_directory(Some("Fonts"), dirs::font_dir());
		plugin.add_directory(Some("Home"), dirs::home_dir());
		plugin.add_directory(None, dirs::picture_dir());
		plugin.add_directory(None, dirs::public_dir());
		plugin.add_directory(None, dirs::template_dir());
		plugin.add_directory(None, dirs::video_dir());

		plugin
	}
}

impl Plugin for DirectoryPlugin {
	fn get_results(&self, query: &str) -> core::Result<Vec<(usize, Box<dyn SearchResult>)>> {
		let query = query.to_lowercase().replace(' ', "");
		Ok(self.results.iter()
			.map(|app| (app.get_ranking(&query), Box::new(app.clone()) as Box<dyn SearchResult>))
			.filter(|(score, _)| *score > 0)
			.collect::<Vec<(usize, Box<dyn SearchResult>)>>()
		)
	}

	fn get_styles(&self) -> core::Result<&'static str> {
		Ok(include_str!("../style/.build.css"))
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
	registrar.register("directory", DirectoryPlugin::new());
}

core::export_plugin!(register);
