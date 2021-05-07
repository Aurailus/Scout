use scout_core::{ Plugin, SearchResult, PluginRegistrar };

mod result;
use result::{WebSearchResult, SearchEngine };

pub struct WebSearchPlugin {
	engines: Vec<SearchEngine>
}

impl WebSearchPlugin {
	fn new() -> Box<dyn Plugin> {
		gtk::init().unwrap();

		let engines = vec![SearchEngine {
			name: "Google".to_string(),
			base_url: "https://google.com/search".to_string(),
			query_argument: "q".to_string(),
			icon: None
		}];

		Box::new(WebSearchPlugin { engines })
	}
}

impl Plugin for WebSearchPlugin {
	fn get_results(&self, query: &str) -> scout_core::Result<Vec<(usize, Box<dyn SearchResult>)>> {
		if query.len() == 0 {
			return Ok(vec![]);
		}

		Ok(self.engines.iter()
			.map(|engine| Box::new(WebSearchResult::new(engine, query)) as Box<dyn SearchResult>)
			.map(|result| (result.get_ranking(&query), result))
			.filter(|(score, _)| *score > 0)
			.collect::<Vec<(usize, Box<dyn SearchResult>)>>())
	}

	fn get_styles(&self) -> scout_core::Result<&'static str> {
		Ok(include_str!("../style/.build.css"))
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
	registrar.register("websearch", WebSearchPlugin::new());
}

scout_core::export_plugin!(register);
