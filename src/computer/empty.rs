//! What the list of the map says when it has no rows: nothing is charted yet, or the search
//! finds nothing.

/// What the list says while nothing is charted and no search is typed.
pub const NOTHING: &str = "Nothing charted here yet. Visit a place to chart it.";

/// Returns the sentence the empty list shows for a table of `count` rows and the search
/// `filter`, or nothing while the list has rows.
pub fn empty(count: usize, filter: &str) -> String {
    match (count, filter.is_empty()) {
        (0, true) => NOTHING.to_string(),
        (0, false) => format!("No place is called \"{filter}\". Press Esc to clear the search."),
        _ => String::new(),
    }
}
