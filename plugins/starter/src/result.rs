use gtk::prelude::*;

use scout_core::SearchResult;

/** A search result. */
#[derive(Clone)]
pub struct StarterResult {
	widget: gtk::Box,
	top_button: gtk::Button
}

impl StarterResult {
	/** Creates the result widget and the search result itself. */
	pub fn new() -> Self {
		let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
		widget.get_style_context().add_class("Starter");
		widget.set_widget_name("SearchResult");

		let top_button = gtk::Button::with_label("Replace Me");
		top_button.get_style_context().add_class("flat");
		widget.pack_start(&top_button, true, true, 0);

		StarterResult { widget, top_button }
	}
}

impl SearchResult for StarterResult {
	/** Returns a search ranking for the specified query. */
	fn get_ranking(&self, _query: &str) -> usize {
		15
	}

	/** Called when the result is the first, its top button should be made not focusable. */
	fn set_first(&self, first: bool) -> () {
		self.top_button.set_can_focus(!first);
	}

	/** Should trigger the main action of the search result. */
	fn activate(&self) {
		println!("Activate");
	}

	/** Returns the result widget. */
	fn get_result_widget(&self) -> gtk::Widget {
		self.widget.clone().upcast()
	}

	/** Returns the preview widget. This can be generated here as it is only called when the result is focused. */
	fn get_preview_widget(&self) -> gtk::Widget {
		let widget = gtk::Box::new(gtk::Orientation::Vertical, 4);
		widget.get_style_context().add_class("Starter");
		widget.set_widget_name("SearchPreview");
		widget.set_border_width(12);

		let label = gtk::Label::new(Some("Replace me"));
		widget.pack_start(&label, false, false, 4);

		widget.upcast()
	}
}
