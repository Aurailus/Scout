
/**
 * Base search result trait.
 */

pub trait SearchResult {

	/**
	 * Returns the score of the result, as determined by the plugin.
	 * Higher scores indicate a higher relevance.
	 */

	fn get_score(&self) -> usize;


	/**
	 * Indicates that this result is the first result displayed,
	 * which may trigger special focus / display behavior.
	 * The first result's primary button should not be focusable,
	 * it will instead be triggered with the `activate` method.
	 */

	fn set_first(&self, first: bool) -> ();


	/**
	 * Triggers the primary action of the result widget.
	 * This is triggered on the first result when activating the search entry.
	 */

	fn activate(&self) -> ();


	/**
	 * Returns a widget representing the result in the results pane.
	 */

	fn get_result_widget(&self) -> gtk::Widget;


	/**
	 * Returns a widget representing the result in the preview pane.
	 */

	fn get_preview_widget(&self) -> gtk::Widget;
}
