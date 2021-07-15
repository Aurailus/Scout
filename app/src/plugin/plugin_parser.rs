/**! Parses plugins and stores them, providing methods to retrieve data from them. */

use std::rc::Rc;
use scout_core::{ Shared, SearchResult };

use crate::app::App;
use super::plugin_bindings::PluginBindings;

#[derive(Default)]
pub struct PluginParser {
	plugins: Vec<Shared<Box<dyn scout_core::Plugin>>>,
	_bindings: Vec<Shared<Box<dyn scout_core::PluginBindings>>>
}

impl PluginParser {
	pub fn new() -> Shared<PluginParser> { Shared::new(PluginParser::default()) }

	/**
	 * Attempts to load a plugin at the specified path.
	 * Returns a result indicating success.
	 */

	pub fn load<P: AsRef<std::ffi::OsStr>>(&mut self, app: &Shared<App>, library_path: P)
		-> std::io::Result<()> {
		unsafe {
			let library = Rc::new(libloading::Library::new(library_path)
				.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?);

			let decl = library.get::<*mut scout_core::PluginDeclaration>(b"PLUGIN_DECLARATION\0")
				.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?.read();

			if decl.rustc_version != scout_core::RUSTC_VERSION || decl.core_version != scout_core::CORE_VERSION {
				return Err(std::io::Error::new(std::io::ErrorKind::Other, "Plugin Version Mismatch.")); }

			let bindings: Shared<Box<dyn scout_core::PluginBindings>> =
				Shared::new(Box::new(PluginBindings::new(app.clone(), Rc::clone(&library))));

			(decl.register)(bindings.clone());

			let bindings_borrow = bindings.borrow();
			match bindings_borrow.get_plugin() {
				Ok((identifier, plugin)) => {
					println!("Registered plugin '{}'.", identifier);
					self.plugins.push(plugin);
					drop(bindings_borrow);
					self._bindings.push(bindings);
					Ok(())
				},
				Err(err) => {
					println!("Failed to register plugin: {:?}", err);
					Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to register"))
				}
			}
		}
	}

	/**
	 * Calls a plugin by name,
	 * Returns a result with data or an error.
	 */

	pub fn get_results(&self, query: &str) -> Vec<Box<dyn SearchResult>> {
		let mut results = vec![];

		for plugin in self.plugins.iter() {
			let res = scout_core::or_continue!(plugin.borrow().get_results(query));
			results.extend(res);
		}

		results.retain(|result| result.get_score() > 0);
		results.sort_by(|result_a, result_b| result_b.get_score().partial_cmp(&result_a.get_score()).unwrap());
		let min = if results.len() >= 1 { (results[0].get_score() as f64 * 0.75) as usize } else { 0 };
		results.retain(|result| result.get_score() >= min);

		results
	}
}
