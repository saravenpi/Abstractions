//! Narrowing and ordering the list: the filter typed on the keyboard and the sort order.

use super::rows::Line;

/// Most characters the filter holds.
pub const FILTER_CHARS: usize = 16;

/// How the rows of the list are ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// As the table is built: outward from the star in a system, nearest first elsewhere.
    #[default]
    Near,
    /// By name, from A to Z.
    Name,
}

impl Sort {
    /// Returns the other order.
    pub fn next(self) -> Sort {
        match self {
            Sort::Near => Sort::Name,
            Sort::Name => Sort::Near,
        }
    }

    /// Returns the name of the order as the header shows it.
    pub fn label(self) -> &'static str {
        match self {
            Sort::Near => "by distance",
            Sort::Name => "by name",
        }
    }
}

/// Adds the letters of `text` to `filter`, up to [`FILTER_CHARS`], and returns true when the
/// filter grew. Anything that is not a letter is left out, so digits stay free for other keys.
pub fn type_into(filter: &mut String, text: &str) -> bool {
    let room = FILTER_CHARS.saturating_sub(filter.chars().count());
    let letters = text.chars().filter(|c| c.is_alphabetic()).take(room);
    let before = filter.len();
    filter.extend(letters.flat_map(char::to_lowercase));
    filter.len() > before
}

/// Returns true when `title` holds `filter`, whatever the case. An empty filter holds for all.
pub fn matches(title: &str, filter: &str) -> bool {
    title.to_lowercase().contains(&filter.to_lowercase())
}

/// Returns the rows whose name holds `filter`, in the order `sort` asks for.
pub fn arrange(mut lines: Vec<Line>, filter: &str, sort: Sort) -> Vec<Line> {
    lines.retain(|line| matches(&line.title, filter));
    if sort == Sort::Name {
        lines.sort_by_key(|line| line.title.to_lowercase());
    }
    lines
}

/// Returns the line of the header that names the list: which rows are drawn, the order, and
/// the filter when there is one. An empty list says what to do about it.
pub fn summary(
    heading: &str,
    (first, last, count): (usize, usize, usize),
    view: (&str, Sort),
) -> String {
    let (filter, sort) = view;
    match (count, filter.is_empty()) {
        (0, true) => format!("{heading} - nothing charted yet. Visit a system to chart more"),
        (0, false) => format!("No name holds \"{filter}\". Backspace edits it, Esc clears it"),
        (_, true) => format!("{heading} - {first} to {last} of {count} {}", sort.label()),
        (_, false) => format!(
            "\"{filter}\" - {first} to {last} of {count} {}",
            sort.label()
        ),
    }
}
