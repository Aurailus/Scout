use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

use freedesktop_entry_parser::parse_entry;

use scout_core::{ Plugin, SearchResult, PluginRegistrar };

mod result;
use result::{ Action, ApplicationResult };

pub struct ApplicationPlugin {
	results: Vec<ApplicationResult>
}

impl ApplicationPlugin {
	fn find_applications() -> Vec<ApplicationResult> {
		let mut search_paths = env::var("XDG_DATA_DIRS")
			.and_then(|string| Ok(string.split(":").map(|string| format!("{}/applications", string).into()).collect::<Vec<PathBuf>>()))
			.unwrap_or_else(|_| vec![]);
		if let Some(dir) = dirs::data_dir() { search_paths.push(format!("{}/applications", dir.to_str().unwrap()).into()); }
		else { search_paths.push(format!("/home/{}/.local/share/applications", &whoami::username()).into()); }
		
		let mut found = Vec::<ApplicationResult>::new();

		while search_paths.len() != 0 {
			let path = search_paths.pop().unwrap();
			let dir_iter = scout_core::or_continue!(std::fs::read_dir(&path));
			
			for entry in dir_iter {
				let entry = scout_core::or_continue!(entry);
				let path = entry.path();
				
				if path.is_dir() {
					search_paths.push(path);
					continue;
				}

				if path.extension() == Some(OsStr::new("desktop")) {
					let parsed = scout_core::or_continue!(parse_entry(path));
					let entry = parsed.section("Desktop Entry");

					let show = entry.attr("NoDisplay").unwrap_or("false") == "false" && entry.attr("Hidden").unwrap_or("false") == "false";

					let action_names = entry.attr("Actions").and_then(|s| Some(s.split(';')
						.filter(|s| !s.is_empty()).collect())).unwrap_or_else(|| vec![]);
					let actions = if action_names.len() > 0 {
						Some(action_names.iter().map(|name| {
							let entry = parsed.section(["Desktop Action", name].join(" "));
							Action {
								name: entry.attr("Name").unwrap_or("Unnamed Action").to_owned(),
								exec: entry.attr("Exec").unwrap().to_owned(),
							}
						}).collect())
					} else { None };
					
					let exec = entry.attr("Exec");
					if exec.is_none() { continue; }

					if show {
						found.push(ApplicationResult::new(
							entry.attr("Name").unwrap_or("Unnamed Application"),
							entry.attr("Comment").unwrap_or(""),
							&ApplicationResult::choose_category(entry.attr("Categories")),
							exec.unwrap(),
							entry.attr("Icon"),
							actions
						))
					}
				}
			}
		}

		found.sort();
		found.dedup();
		found
	}

	fn new() -> Box<dyn Plugin> {
		gtk::init().unwrap();
		
		Box::new(ApplicationPlugin {
			results: ApplicationPlugin::find_applications()
		})
	}
}

impl Plugin for ApplicationPlugin {
	fn get_results(&self, query: &str) -> scout_core::Result<Vec<(usize, Box<dyn SearchResult>)>> {
		let query = query.to_lowercase().replace(' ', "");
		Ok(self.results.iter().map(|app| (app.get_ranking(&query), Box::new(app.clone()) as Box<dyn SearchResult>))
			.filter(|(score, _)| *score > 0).collect::<Vec<(usize, Box<dyn SearchResult>)>>())
	}

	fn get_styles(&self) -> scout_core::Result<&'static str> {
		Ok(include_str!("../style/.build.css"))
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
	registrar.register("application", ApplicationPlugin::new());
}

scout_core::export_plugin!(register);
