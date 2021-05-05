use scout_core::{ Plugin, PluginRegistrar };

pub struct Program;

impl Plugin for Program {
	fn call(&self, _args: &[f64]) -> Result<f64, scout_core::InvocationError> {
		println!("Plugin loaded ZZZ");
		Ok(102.0)
	}
}

#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
	registrar.register("program", Box::new(Program));
}

scout_core::export_plugin!(register);
