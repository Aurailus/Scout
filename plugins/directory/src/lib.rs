use core::{ Plugin, SearchResult, PluginBindings, Shared };

use std::path::PathBuf;

mod result;
use result::{ DirectoryResult };

pub struct DirectoryPlugin {
	bindings: Shared<Box<dyn PluginBindings>>,
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

	fn new(bindings: Shared<Box<dyn PluginBindings>>) -> Box<dyn Plugin> {
		gtk::init().unwrap();

		let mut plugin = Box::new(DirectoryPlugin {
			bindings,
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
	fn get_results(&self, query: &str) -> core::Result<Vec<Box<dyn SearchResult>>> {
		let query = query.to_lowercase().replace(' ', "");
		Ok(self.results.iter()
			.map(|res| {
				let mut result = res.clone();
				result.set_score_from_query(&query);
				Box::new(result) as Box<dyn SearchResult>
			})
			.filter(|result| result.get_score() > 0)
			.collect::<Vec<Box<dyn SearchResult>>>()
		)

		// Ok(self.results.iter()
		// 	.map(|app| (app.get_ranking(&query), Box::new(app.clone()) as Box<dyn SearchResult>))
		// 	.filter(|(score, _)| *score > 0)
		// 	.collect::<Vec<(usize, Box<dyn SearchResult>)>>()
		// )
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(bindings_shr: Shared<Box<dyn PluginBindings>>) {
	let mut bindings = bindings_shr.borrow_mut();
	let plugin = DirectoryPlugin::new(bindings_shr.clone());
	bindings.add_stylesheet(include_str!("../style/.build.css"));
	bindings.register("directory", plugin);
}

core::export_plugin!(register);
