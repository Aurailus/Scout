/*!
 * Manages loading dynamic plugins for Scout.
 * Adapted from https://adventures.michaelfbryan.com/posts/plugins-in-rust/.
 */

use std::rc::Rc;
use std::collections::HashMap;

use scout_core::{ Plugin, SearchResult, Result, Shared };

use super::window::App;

/** Proxy for a Plugin that keeps it's Library loaded. */
struct PluginProxy {
	plugin: Box<dyn Plugin>,
	_lib: Rc<libloading::Library>
}

impl Plugin for PluginProxy {
	fn get_results(&self, query: &str) -> Result<Vec<(usize, Box<dyn SearchResult>)>> {
		self.plugin.get_results(query)
	}

	// fn get_styles(&self) -> Result<&'static str> {
	// 	self.plugin.get_styles()
	// }
}


/**
 * Allows a plugin to access the application.
 */

struct PluginBindings {
	app: Shared<App>,
	lib: Rc<libloading::Library>,
	plugins: HashMap<String, PluginProxy>
}

impl PluginBindings {
	fn new(app: Shared<App>, lib: Rc<libloading::Library>) -> PluginBindings {
		PluginBindings { app, lib, plugins: HashMap::new() }
	}
}

impl scout_core::PluginBindings for PluginBindings {
	fn register(&mut self, name: &str, plugin: Box<dyn Plugin>) {
		let proxy = PluginProxy { plugin, _lib: Rc::clone(&self.lib) };
		self.plugins.insert(name.to_owned(), proxy);
	}

	fn add_stylesheet(&mut self, stylesheet: &'static str) {
		println!("{}", stylesheet);
	}
}


/**
 * The main Plugin store. Contains all loaded libraries and plugins,
 * and has methods to interact with them.
 */

#[derive(Default)]
pub struct PluginParser {
	libraries: Vec<Rc<libloading::Library>>,
}

impl PluginParser {
	pub fn new() -> PluginParser { PluginParser::default() }

	/**
	 * Attempts to load a plugin at the specified path.
	 * Returns a result containing the plugins loaded, or an error.
	 */

	pub fn load<P: AsRef<std::ffi::OsStr>>(&mut self, app: Shared<App>, library_path: P)
		-> std::io::Result<()> {
		unsafe {
			let library = Rc::new(libloading::Library::new(library_path)
				.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?);

			let decl = library.get::<*mut scout_core::PluginDeclaration>(b"PLUGIN_DECLARATION\0")
				.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?.read();

			if decl.rustc_version != scout_core::RUSTC_VERSION || decl.core_version != scout_core::CORE_VERSION {
				return Err(std::io::Error::new(std::io::ErrorKind::Other, "Plugin Version Mismatch.")); }

			let registrar: Shared<Box<dyn scout_core::PluginBindings>> =
				Shared::new(Box::new(PluginBindings::new(app.clone(), Rc::clone(&library))));

			(decl.register)(registrar.clone());

			self.libraries.push(library);

			Ok(())
		}
	}

	/**
	 * Returns a vector of string slices containing custom plugin CSS.
	 */

	pub fn get_styles(&self) -> Vec<&'static str> {
		// self.plugins.iter().map(|(_, p)| p.get_styles()).filter(|p| p.is_ok()).map(|p| p.unwrap()).collect::<_>()
		vec![]
	}

	/**
	 * Calls a plugin by name,
	 * Returns a result with data or an error.
	 */

	pub fn get_results(&self, query: &str) -> Vec<(usize, Box<dyn SearchResult>)> {
		let mut results = vec![];

		// for tuple in self.plugins.iter() {
		// 	let res = scout_core::or_continue!(tuple.1.get_results(query));
		// 	results.extend(res);
		// }

		results
	}
}
