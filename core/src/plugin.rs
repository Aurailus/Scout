use crate::SearchResult;

/**
 * Represents an error in invoking a plugin method.
 */

#[derive(Debug)]
pub enum InvocationError {
	
	/** Used when the plugin does not implement the feature requested. */
	DoesNotProvide { feature: String },

	/** Generic error. */
	Other { msg: String }
}

impl<S: ToString> From<S> for InvocationError {
	fn from(other: S) -> InvocationError {
		InvocationError::Other {
			msg: other.to_string(),
		}
	}
}


/**
 * Generic result type.
 */

pub type Result<T> = std::result::Result<T, InvocationError>;


/**
 * Base trait that all Plugins should implement.
 */

pub trait Plugin {

	/**
	 * Returns a list of probable results for the inputted query,
	 * paired with their search ranking. Results do not need to be sorted,
	 * the main app will do that once it collects other Plugins' results.
	 *
	 * - `query` - The search query, transformed to ascii-lowercase.
	 */
	
	fn get_results(&self, query: &str) -> Result<Vec<(usize, Box<dyn SearchResult>)>>;
}
