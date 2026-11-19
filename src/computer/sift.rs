//! Narrowing and ordering the list: the filter typed on the keyboard and the sort order.

use super::format::clip;
use super::layout::LINE_CHARS;
use super::route::Target;
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

    /// Returns the name of the order as the header shows it: nothing for the order the table
    /// is built in.
    pub fn label(self) -> &'static str {
        match self {
            Sort::Near => "",
            Sort::Name => " by name",
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

/// Returns true when the rows `a` and `b` are a planet and one of its moons, either way.
fn kin(a: &Line, b: &Line) -> bool {
    match (a.target, b.target) {
        (Target::Body(x), Target::Body(y)) => {
            x != y && x.planet() == y.planet() && !(x.is_moon() && y.is_moon())
        }
        _ => false,
    }
}

/// Returns what a row is sorted on by name: the name of its planet for a moon, then whether
/// it is a moon, then its own name, so that each moon follows its planet.
fn key(lines: &[Line], line: &Line) -> (String, bool, String) {
    let own = line.title.to_lowercase();
    let moon = matches!(line.target, Target::Body(id) if id.is_moon());
    let planet = lines.iter().find(|other| moon && kin(line, other));
    let family = planet.map_or_else(|| own.clone(), |planet| planet.title.to_lowercase());
    (family, moon, own)
}

/// Returns the rows whose name holds `filter`, in the order `sort` asks for. A planet stays
/// with its moons: the filter keeps a moon when its planet's name holds it and a planet when
/// one of its moons' does.
pub fn arrange(lines: Vec<Line>, filter: &str, sort: Sort) -> Vec<Line> {
    let kept = |line: &Line| {
        let near = |other: &Line| kin(line, other) && matches(&other.title, filter);
        matches(&line.title, filter) || lines.iter().any(near)
    };
    let keep: Vec<bool> = lines.iter().map(kept).collect();
    let keys: Vec<_> = lines.iter().map(|line| key(&lines, line)).collect();
    let mut rows: Vec<_> = keys.into_iter().zip(lines).zip(keep).collect();
    rows.retain(|(_, keep)| *keep);
    if sort == Sort::Name {
        rows.sort_by(|a, b| a.0.0.cmp(&b.0.0));
    }
    rows.into_iter().map(|((_, line), _)| line).collect()
}

/// Returns the line of the header that names the list: the filter or the heading, which rows
/// are drawn, and the order when it is by name. It fits one line of the side panel: a long
/// heading is cut short.
pub fn summary(
    heading: &str,
    (first, last, count): (usize, usize, usize),
    view: (&str, Sort),
) -> String {
    let (filter, sort) = view;
    let rows = format!(" - {first}-{last} of {count}{}", sort.label());
    let (head, tail) = match (count, filter.is_empty()) {
        (0, true) => (heading.to_string(), " - nothing charted yet".to_string()),
        (0, false) => (format!("No name holds \"{filter}\""), String::new()),
        (_, true) => (heading.to_string(), rows),
        (_, false) => (format!("\"{filter}\""), rows),
    };
    let room = LINE_CHARS.saturating_sub(tail.chars().count());
    format!("{}{tail}", clip(&head, room))
}
