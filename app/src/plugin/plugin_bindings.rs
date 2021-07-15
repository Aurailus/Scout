/**! Provides methods to give a plugin access to the application. */

use std::rc::Rc;
use scout_core::{ Shared, Plugin, InvocationError };

use crate::app::App;
// use super::plugin_proxy::PluginProxy;

pub struct PluginBindings {
	app: Shared<App>,
	identifier: String,
	plugin: Option<Shared<Box<dyn Plugin>>>,
	_lib: Rc<libloading::Library>
}

impl PluginBindings {
	pub fn new(app: Shared<App>, _lib: Rc<libloading::Library>) -> PluginBindings {
		PluginBindings { app, _lib, plugin: None, identifier: "".to_owned() }
	}
}

impl scout_core::PluginBindings for PluginBindings {
	fn register(&mut self, identifier: &str, plugin: Box<dyn Plugin>) {
		if self.plugin.is_some() {
			println!("[WARN] Plugin called register twice, '{}' -> '{}'.", self.identifier, identifier);
		}

		self.identifier = identifier.to_owned();
		self.plugin = Some(Shared::new(plugin));
	}

	fn add_stylesheet(&mut self, stylesheet: &'static str) {
		self.app.borrow_mut().add_stylesheet(stylesheet);
	}

	fn get_plugin(&self) -> Result<(&str, Shared<Box<dyn Plugin>>), InvocationError> {
		if self.plugin.is_some() {
			Ok((&self.identifier, self.plugin.as_ref().unwrap().clone()))
		}
		else {
			Err(InvocationError::RegistrationFailed)
		}
	}
}
