use scout_core::{ Shared, SearchResult };

use super::window::{ Window, WindowCallbacks };

pub struct AppCallbacks {
	pub on_search: Box<dyn FnMut(&str) -> Vec<Box<dyn SearchResult>>>
}

impl Default for AppCallbacks {
	fn default() -> Self {
		AppCallbacks {
			on_search: Box::new(|_| vec![])
		}
	}
}

pub struct App {
	stylesheets: Vec<&'static str>,
	window: Option<Shared<Window>>,
	callbacks: AppCallbacks
}

impl App {
	pub fn new() -> Shared<Self> {
		Shared::new(App {
			window: None,
			stylesheets: vec![],
			callbacks: AppCallbacks::default()
		})
	}

	pub fn add_stylesheet(&mut self, stylesheet: &'static str) {
		self.stylesheets.push(stylesheet);
	}

	pub fn bind(&mut self, callbacks: AppCallbacks) {
		self.callbacks = callbacks;
	}

	pub fn init(&mut self, gtk: &gtk::Application, bind: &Shared<Self>) {
		self.window = Some(Window::new(gtk, &self.stylesheets));

		let bind_a = bind.clone();
		let bind_b = bind.clone();
		self.window.as_ref().unwrap().borrow_mut().bind(WindowCallbacks {
			on_input: Box::new(move |v| bind_a.borrow_mut().on_input(v)),
			on_submit: Box::new(move || bind_b.borrow_mut().on_submit())
		});
		self.stylesheets.clear();
	}

	fn on_input(&mut self, value: &str) {
		let results = (self.callbacks.on_search)(value);
		self.window.as_ref().unwrap().borrow_mut().set_results(results);
	}

	fn on_submit(&mut self) {
		println!("submit");
	}
}
