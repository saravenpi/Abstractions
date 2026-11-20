//! Worlds menu tests: relative times, places, and what the table shows in each mode.

use super::act::Act;
use super::modal::enter_choice;
use super::rows::{Part, cell_text};
use super::when::{place_name, relative};
use super::{Listed, Mode};
use crate::saves::{Place, WorldEntry, WorldSave};
use crate::ui::Tone;
use crate::universe::Universe;

/// Relative times change wording exactly at each boundary.
#[test]
fn relative_times_change_at_each_boundary() {
    let now = 2_000_000_000;
    let cases = [
        (0, "just now"),
        (59, "just now"),
        (60, "1 minute ago"),
        (119, "1 minute ago"),
        (120, "2 minutes ago"),
        (3599, "59 minutes ago"),
        (3600, "1 hour ago"),
        (7200, "2 hours ago"),
        (86_399, "23 hours ago"),
        (86_400, "yesterday"),
        (172_799, "yesterday"),
        (172_800, "2 days ago"),
        (604_799, "6 days ago"),
        (604_800, "1 week ago"),
        (1_209_600, "2 weeks ago"),
        (2_591_999, "4 weeks ago"),
        (2_592_000, "1 month ago"),
        (31_535_999, "12 months ago"),
        (31_536_000, "1 year ago"),
        (63_072_000, "2 years ago"),
    ];
    for (elapsed, shown) in cases {
        assert_eq!(relative(now, now - elapsed), shown, "{elapsed}");
    }
    assert_eq!(relative(now, now + 500), "just now");
    assert_eq!(relative(now, 0), "never");
}

/// A world without a saved place is on the home planet; a saved place names its planet and star.
#[test]
fn places_name_the_planet_and_the_system() {
    let universe = Universe::new(9);
    let mut save = WorldSave {
        seed: 9,
        ..WorldSave::default()
    };
    let star = universe.system(universe.home().star).star.name;
    let home = format!("{}, {star}", universe.planet(universe.home()).name);
    assert_eq!(place_name(&save), home);
    let mut body = universe.home();
    save.place = Some(Place {
        body,
        ..Place::default()
    });
    assert_eq!(place_name(&save), home);
    body.index = 0;
    save.place = Some(Place {
        body,
        ..Place::default()
    });
    let inner = universe.planet(body).name;
    assert_ne!(format!("{inner}, {star}"), home);
    assert_eq!(place_name(&save), format!("{inner}, {star}"));
}

/// Returns a listed world played two hours before the stamp 8200.
fn listed() -> Listed {
    Listed {
        entry: WorldEntry {
            id: "isle".to_string(),
            save: WorldSave {
                name: "Isle".to_string(),
                seed: 42,
                play_seconds: 5040.0,
                last_played: 1000,
                ..WorldSave::default()
            },
        },
        place: "Kolva, Varena".to_string(),
    }
}

/// A row shows the name, the place, the seed, the play time and the relative time.
#[test]
fn a_row_shows_every_fact_of_its_world() {
    let world = listed();
    let shown = |part| cell_text(&world, part, None, 8200);
    assert_eq!(shown(Part::Name), ("Isle".to_string(), Tone::Plain));
    assert_eq!(shown(Part::Place).0, "Kolva, Varena");
    assert_eq!(shown(Part::Seed).0, "42");
    assert_eq!(shown(Part::Played).0, "1 h 24 min");
    assert_eq!(shown(Part::Last).0, "2 hours ago");
}

/// The selected row shows the typed name or the delete question in place, and Enter deletes
/// only once that choice is armed.
#[test]
fn the_selected_row_shows_the_mode_in_place() {
    let world = listed();
    let shown = |part, mode: &Mode| cell_text(&world, part, Some(mode), 8200);
    let typing = Mode::Rename("Ato".to_string());
    assert_eq!(shown(Part::Name, &typing).0, "Ato_");
    assert_eq!(shown(Part::Name, &typing).1, Tone::Accent);
    let asking = Mode::Delete { armed: false };
    assert_eq!(shown(Part::Name, &asking).0, "Delete Isle?");
    assert_eq!(shown(Part::Name, &asking).1, Tone::Danger);
    assert_eq!(shown(Part::Seed, &asking).0, "42");
    assert_eq!(enter_choice(false), Act::Cancel);
    assert_eq!(enter_choice(true), Act::Confirm);
}
