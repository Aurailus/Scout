use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

use freedesktop_entry_parser::parse_entry;

use scout_core::{ Plugin, SearchResult, PluginBindings, Shared };

mod result;
use result::{ Action, ApplicationResult };

pub struct ApplicationPlugin {
	bindings: Shared<Box<dyn PluginBindings>>,
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

	fn new(bindings: Shared<Box<dyn PluginBindings>>) -> Box<dyn Plugin> {
		gtk::init().unwrap();

		Box::new(ApplicationPlugin {
			bindings,
			results: ApplicationPlugin::find_applications()
		})
	}
}

impl Plugin for ApplicationPlugin {
	fn get_results(&self, query: &str) -> scout_core::Result<Vec<Box<dyn SearchResult>>> {
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
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(bindings_shr: Shared<Box<dyn PluginBindings>>) {
	let mut bindings = bindings_shr.borrow_mut();
	let plugin = ApplicationPlugin::new(bindings_shr.clone());
	bindings.add_stylesheet(include_str!("../style/.build.css"));
	bindings.register("application", plugin);
}

scout_core::export_plugin!(register);
