use scout_core::{ Plugin, SearchResult, PluginRegistrar };

mod result;
use result::StarterResult;

/** Your main plugin struct. */
pub struct StarterPlugin {
	results: Vec<StarterResult>
}

impl StarterPlugin {
	/** Initialize the plugin, as well as the results it can query. */
	fn new() -> Box<dyn Plugin> {
		gtk::init().unwrap();
		
		Box::new(StarterPlugin {
			results: vec![ StarterResult::new() ]
		})
	}
}

impl Plugin for StarterPlugin {
	/** Called when the user searches. */
	fn get_results(&self, query: &str) -> scout_core::Result<Vec<(usize, Box<dyn SearchResult>)>> {
		Ok(self.results.iter()
			.map(|app| (app.get_ranking(&query), Box::new(app.clone()) as Box<dyn SearchResult>))
			.filter(|(score, _)| *score > 0)
			.collect::<Vec<(usize, Box<dyn SearchResult>)>>()
		)
	}

	/** Called when the app is initialized, can return custom CSS. */
	fn get_styles(&self) -> scout_core::Result<&'static str> {
		Ok(include_str!("../style/.build.css"))
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
	registrar.register("starter", StarterPlugin::new());
}

scout_core::export_plugin!(register);
