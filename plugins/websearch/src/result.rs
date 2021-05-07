use gtk::prelude::*;
use scout_core::SearchResult;
use url::{ Url };


#[derive(Clone)]
pub struct SearchEngine {
	pub(crate) name: String,
	pub(crate) base_url: String,
	pub(crate) query_argument: String,
	pub(crate) icon: Option<String>,
}


/**
 * A web search engine result, created from a search engine.
 * Opens up the search engine in the web browser when activated.
 */

#[derive(Clone)]
pub struct WebSearchResult {
	engine: SearchEngine,
	query: String,
	widget: gtk::Box,
	top_button: gtk::Button
}


impl WebSearchResult {
	/**
	 * Creates a new SearchEngine result, with a corresponding result widget.
	 */
	pub fn new(engine: &SearchEngine, query: &str) -> Self {
		let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
		widget.get_style_context().add_class("Program");
		widget.set_widget_name("SearchResult");
		let top_button = gtk::Button::new();

		{
			top_button.get_style_context().add_class("flat");
			widget.pack_start(&top_button, true, true, 0);

			top_button.connect_clicked(move |_| {
				// TODO: implement this
			});

			let widget_top = gtk::Box::new(gtk::Orientation::Horizontal, 4);
			top_button.add(&widget_top);

			let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
			icon_box.get_style_context().add_class("IconBox");
			widget_top.pack_start(&icon_box, false, false, 4);

			let icon = gtk::Image::from_icon_name(engine.icon.as_deref(), gtk::IconSize::Dnd);
			icon.set_size_request(32, 32);
			icon_box.pack_start(&icon, false, false, 0);

			let description_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
			widget_top.pack_start(&description_box, true, true, 0);

			let category_label = gtk::Label::new(Some(&[ "<span size='small' weight='bold'>", &engine.name, "</span>" ].join("")));
			category_label.get_style_context().add_class("Category");
			category_label.set_ellipsize(pango::EllipsizeMode::End);
			category_label.set_use_markup(true);
			category_label.set_xalign(0.0);
			description_box.pack_start(&category_label, false, false, 1);

			let label = gtk::Label::new(Some(query));
			label.set_ellipsize(pango::EllipsizeMode::End);
			label.set_xalign(0.0);
			description_box.pack_start(&label, false, false, 1);
		}

		WebSearchResult {
			engine: engine.clone(),
			query: query.to_owned(),
			top_button,
			widget
		}
	}
}

impl SearchResult for WebSearchResult {
	fn get_ranking(&self, query: &str) -> usize {
		query.len() * 5 + 1
	}

	fn set_first(&self, first: bool) -> () {
		self.top_button.set_can_focus(!first);
	}

	fn activate(&self) {
		let mut url = Url::parse(self.engine.base_url.as_str()).unwrap();
		url.query_pairs_mut().append_pair(self.engine.query_argument.as_str(), self.query.as_str());

		match opener::open(url.as_str()) {
			Ok(_) => {}
			Err(e) => println!("{}", e)
		}
	}

	fn get_result_widget(&self) -> gtk::Widget {
		self.widget.clone().upcast()
	}

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
