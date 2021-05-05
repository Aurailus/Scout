/*!
 * Manages loading dynamic plugins for Scout.
 * Adapted from https://adventures.michaelfbryan.com/posts/plugins-in-rust/.
 */

use std::rc::Rc;
use std::collections::{ HashMap, HashSet };

use scout_core::{ Plugin, InvocationError };


/** Proxy for a Plugin that keeps it's Library loaded. */
struct PluginProxy {
	plugin: Box<dyn Plugin>,
	_lib: Rc<libloading::Library>,
}

impl Plugin for PluginProxy {
	fn call(&self, args: &[f64]) -> Result<f64, scout_core::InvocationError> {
		self.plugin.call(args)
	}
}


/** Passed into a library to allow it to register plugins. */
struct PluginRegistrar {
	plugins: HashMap<String, PluginProxy>,
	lib: Rc<libloading::Library>
}

impl PluginRegistrar {
	fn new(lib: Rc<libloading::Library>) -> PluginRegistrar {
		PluginRegistrar { lib, plugins: HashMap::default() }
	}
}

impl scout_core::PluginRegistrar for PluginRegistrar {
	fn register(&mut self, name: &str, plugin: Box<dyn Plugin>) {
		let proxy = PluginProxy { plugin, _lib: Rc::clone(&self.lib) };
		self.plugins.insert(name.to_owned(), proxy);
	}
}


/**
 * The main Plugin store. Contains all loaded libraries and plugins,
 * and has methods to interact with them.
 */

#[derive(Default)]
pub struct Plugins {
	plugins: HashMap<String, PluginProxy>,
	libraries: Vec<Rc<libloading::Library>>,
}

impl Plugins {
	pub fn new() -> Plugins { Plugins::default() }

	/**
	 * Attempts to load a plugin at the specified path.
	 * Returns a result containing the plugins loaded, or an error.
	 */

	pub unsafe fn load<P: AsRef<std::ffi::OsStr>>(&mut self, library_path: P) -> std::io::Result<HashSet<String>> {
		let library = Rc::new(libloading::Library::new(library_path)
			.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?);

		let decl = library.get::<*mut scout_core::PluginDeclaration>(b"PLUGIN_DECLARATION\0")
			.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)))?.read();

		if decl.rustc_version != scout_core::RUSTC_VERSION || decl.core_version != scout_core::CORE_VERSION {
			return Err(std::io::Error::new(std::io::ErrorKind::Other, "Plugin Version Mismatch.")); }

		let mut registrar = PluginRegistrar::new(Rc::clone(&library));
		(decl.register)(&mut registrar);

		let loaded = registrar.plugins.iter().map(|(k, _)| k.clone()).collect::<HashSet<_>>();

		self.plugins.extend(registrar.plugins);
		self.libraries.push(library);

		Ok(loaded)
	}

	/**
	 * Calls a plugin by name,
	 * Returns a result with data or an error.
	 */

	pub fn call(&self, plugin: &str, arguments: &[f64]) -> Result<f64, InvocationError> {
		self.plugins.get(plugin).ok_or_else(|| format!("\"{}\" not found.", plugin))?.call(arguments)
	}
}
