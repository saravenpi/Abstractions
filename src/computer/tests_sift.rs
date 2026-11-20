//! Computer tests: the filter and the order of the list, alone and typed into the open panel.

use bevy::prelude::*;

use super::show::cell_text;
use super::sift::{FILTER_CHARS, arrange, matches, summary, type_into};
use super::tests_rig::{game, run, tap};
use super::view::Part;
use super::*;
use crate::ui::{Icon, Tone};
use crate::universe::{BodyId, GalaxyId, StarId, Universe};

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
            line("Varena", 0.0),
            line("Kolva (home)", 5.0),
            line("arko", 9.0),
        ]
    };
    let names = |lines: Vec<Line>| lines.into_iter().map(|line| line.title).collect::<Vec<_>>();
    assert_eq!(
        names(arrange(rows(), "", Sort::Near)),
        ["Varena", "Kolva (home)", "arko"]
    );
    assert_eq!(
        names(arrange(rows(), "", Sort::Name)),
        ["arko", "Kolva (home)", "Varena"]
    );
    assert_eq!(
        names(arrange(rows(), "KO", Sort::Near)),
        ["Kolva (home)", "arko"]
    );
    assert_eq!(
        names(arrange(rows(), "ko", Sort::Name)),
        ["arko", "Kolva (home)"]
    );
    assert!(arrange(rows(), "zz", Sort::Near).is_empty());
    assert!(matches("Varena", "") && matches("Varena", "REN") && !matches("Varena", "x"));
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
            "Varena system",
            (1, count.min(6), count),
            (filter, Sort::Near),
        )
    };
    assert_eq!(shown(9, ""), "Varena system - 1-6 of 9");
    assert_eq!(shown(2, "ko"), "\"ko\"");
    assert_eq!(shown(0, ""), "None charted yet");
    assert!(shown(0, "zz").starts_with("No name holds \"zz\""));
    let by_name = summary("Known universe", (1, 4, 4), ("", Sort::Name));
    assert_eq!(by_name, "Known universe by name");
}

/// A row wears the icon of what it is, a pin when it is the destination and the ship where
/// the ship is, and a place never visited is dimmed.
#[test]
fn a_row_shows_what_it_is_and_whether_it_was_visited() {
    let star = Line {
        target: Target::Star(StarId::default()),
        ..line("Varena", 0.0)
    };
    let planet = Line {
        target: Target::Body(BodyId::default()),
        visited: true,
        ..line("Kolva", 0.0)
    };
    let mark = |line: &Line, marks| cell_text(line, Part::Mark, marks);
    let (galaxy, plain) = (line("Aru", 0.0), (false, false));
    let cases = [
        (&star, plain, Icon::Star, Tone::Muted),
        (&planet, plain, Icon::Planet, Tone::Plain),
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

/// A search left over once typing went elsewhere, as after naming a favourite, is cleared by
/// Escape, and the next Escape closes the map.
#[test]
fn escape_clears_a_search_left_over() {
    let mut app = game("leftover");
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    tap(&mut app, KeyCode::Slash, None);
    tap(&mut app, KeyCode::KeyK, Some("k"));
    *app.world_mut().resource_mut::<Typing>() = Typing::Off;
    run(&mut app, 2);
    let open = |app: &App| *app.world().resource::<State<Computer>>().get() == Computer::Open;
    assert_eq!(app.world().resource::<Board>().filter, "k");
    tap(&mut app, KeyCode::Escape, None);
    assert!(open(&app) && app.world().resource::<Board>().filter.is_empty());
    tap(&mut app, KeyCode::Escape, None);
    run(&mut app, 2);
    assert!(!open(&app));
}

/// Searches the open panel for the first word of the home planet's generated name, and checks
/// that the list keeps the rows that hold it, the home planet among them.
fn search_home(app: &mut App) {
    let universe = *app.world().resource::<Universe>();
    let home = universe.planet(universe.home()).name;
    let word: String = home.chars().take_while(|c| c.is_alphabetic()).collect();
    let before = app.world().resource::<Board>().clone();
    tap(app, KeyCode::Slash, None);
    for letter in word.chars() {
        tap(app, KeyCode::KeyA, Some(&letter.to_string()));
    }
    let board = app.world().resource::<Board>().clone();
    let wanted = arrange(before.lines, &word, before.sort).len();
    assert_eq!(
        (board.filter, board.lines.len()),
        (word.to_lowercase(), wanted)
    );
    assert!(board.lines.iter().any(|line| line.title.starts_with(&home)));
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
    search_home(&mut app);
    tap(&mut app, KeyCode::Backspace, None);
    for (code, letter) in [
        (KeyCode::KeyZ, "z"),
        (KeyCode::KeyQ, "q"),
        (KeyCode::KeyZ, "z"),
    ] {
        tap(&mut app, code, Some(letter));
    }
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
