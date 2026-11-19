//! Computer tests: the filter and the order of the list, alone and typed into the open panel.

use bevy::prelude::*;

use super::show::cell_text;
use super::sift::{FILTER_CHARS, arrange, matches, summary, type_into};
use super::tests_rig::{game, run, tap};
use super::view::Part;
use super::*;
use crate::ui::{Icon, Tone};
use crate::universe::{BodyId, GalaxyId, StarId};

/// Returns a row named `title`, `distance` metres away.
fn line(title: &str, distance: f64) -> Line {
    Line {
        title: title.to_string(),
        detail: String::new(),
        target: Target::Galaxy(GalaxyId::default()),
        distance,
        visited: false,
        at: default(),
    }
}

/// The filter keeps the rows whose name holds it, whatever the case, and the order by name
/// sorts them from A to Z while the default order leaves them as built.
#[test]
fn the_filter_narrows_the_rows_and_the_order_sorts_them() {
    let rows = || {
        vec![
            line("Vereya", 0.0),
            line("Kodya (home)", 5.0),
            line("arko", 9.0),
        ]
    };
    let names = |lines: Vec<Line>| lines.into_iter().map(|line| line.title).collect::<Vec<_>>();
    assert_eq!(
        names(arrange(rows(), "", Sort::Near)),
        ["Vereya", "Kodya (home)", "arko"]
    );
    assert_eq!(
        names(arrange(rows(), "", Sort::Name)),
        ["arko", "Kodya (home)", "Vereya"]
    );
    assert_eq!(
        names(arrange(rows(), "KO", Sort::Near)),
        ["Kodya (home)", "arko"]
    );
    assert_eq!(
        names(arrange(rows(), "ko", Sort::Name)),
        ["arko", "Kodya (home)"]
    );
    assert!(arrange(rows(), "zz", Sort::Near).is_empty());
    assert!(matches("Vereya", "") && matches("Vereya", "REY") && !matches("Vereya", "x"));
    assert_eq!(
        (Sort::default(), Sort::Near.next(), Sort::Name.next()),
        (Sort::Near, Sort::Name, Sort::Near)
    );
}

/// Only letters are typed into the filter, in lower case and up to its length, and the
/// header says the order, the filter, and what to do when nothing is left.
#[test]
fn the_filter_takes_letters_and_shows_in_the_header() {
    let mut filter = String::new();
    assert!(type_into(&mut filter, "K") && type_into(&mut filter, "o"));
    assert!(!type_into(&mut filter, "0") && !type_into(&mut filter, " ") && filter == "ko");
    assert!(type_into(&mut filter, "abcdefghijklmnopqrstuvwxyz"));
    assert_eq!(filter.chars().count(), FILTER_CHARS);
    assert!(!type_into(&mut filter, "z"));
    let shown = |count: usize, filter: &'static str| {
        summary(
            "Vereya system",
            (1, count.min(6), count),
            (filter, Sort::Near),
        )
    };
    assert_eq!(shown(9, ""), "Vereya system - 1 to 6 of 9 by distance");
    assert_eq!(shown(2, "ko"), "\"ko\" - 1 to 2 of 2 by distance");
    assert!(shown(0, "").starts_with("Vereya system - nothing charted yet"));
    assert!(shown(0, "zz").starts_with("No name holds \"zz\""));
    let by_name = summary("Known universe", (1, 4, 4), ("", Sort::Name));
    assert_eq!(by_name, "Known universe - 1 to 4 of 4 by name");
}

/// A row wears the icon of what it is, a pin when it is the destination and the ship where
/// the ship is, and a place never visited is dimmed.
#[test]
fn a_row_shows_what_it_is_and_whether_it_was_visited() {
    let star = Line {
        target: Target::Star(StarId::default()),
        ..line("Vereya", 0.0)
    };
    let planet = Line {
        target: Target::Body(BodyId::default()),
        visited: true,
        ..line("Kodya", 0.0)
    };
    let mark = |line: &Line, marks| cell_text(line, Part::Mark, marks);
    let (galaxy, plain) = (line("Aru", 0.0), (false, false));
    let cases = [
        (&star, plain, Icon::Star, Tone::Muted),
        (&planet, plain, Icon::Earth, Tone::Plain),
        (&galaxy, plain, Icon::Map, Tone::Muted),
        (&star, (false, true), Icon::Send, Tone::Plain),
        (&star, (true, true), Icon::MapPin, Tone::Accent),
    ];
    for (row, marks, icon, tone) in cases {
        assert_eq!(mark(row, marks), (icon.glyph(), tone));
    }
    let title = |line: &Line, marks| cell_text(line, Part::Title, marks).1;
    assert_eq!(title(&star, (false, false)), Tone::Muted);
    assert_eq!(title(&planet, (false, false)), Tone::Plain);
    assert_eq!(title(&planet, (true, false)), Tone::Accent);
}

/// The search key starts a search, typing then narrows the list, Backspace widens it again,
/// Escape clears the search before it closes the map, and the sort key changes the order.
#[test]
fn typing_in_the_panel_filters_the_list() {
    let mut app = game("filter");
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    let rows = |app: &App| app.world().resource::<Board>().lines.len();
    let open = |app: &App| *app.world().resource::<State<Computer>>().get() == Computer::Open;
    let all = rows(&app);
    assert!(all > 2);
    tap(&mut app, KeyCode::Slash, None);
    for (code, letter) in [
        (KeyCode::KeyK, "k"),
        (KeyCode::KeyO, "o"),
        (KeyCode::KeyD, "d"),
    ] {
        tap(&mut app, code, Some(letter));
    }
    let board = app.world().resource::<Board>().clone();
    assert_eq!((board.filter.as_str(), board.lines.len()), ("kod", 1));
    assert!(board.lines[0].title.starts_with("Kodya"));
    tap(&mut app, KeyCode::Backspace, None);
    tap(&mut app, KeyCode::KeyZ, Some("z"));
    assert_eq!(rows(&app), 0);
    tap(&mut app, KeyCode::Escape, None);
    assert!(open(&app) && rows(&app) == all);
    assert!(app.world().resource::<Board>().filter.is_empty());
    tap(&mut app, KeyCode::F2, None);
    assert_eq!(app.world().resource::<Board>().sort, Sort::Name);
    tap(&mut app, KeyCode::Escape, None);
    run(&mut app, 2);
    assert!(!open(&app));
}
