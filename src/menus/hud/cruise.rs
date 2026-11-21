//! The words of the orbital cruise on the prompt line: that a cruise can start and with which
//! key, how far the target is and how long it takes, the glide down, and why a cruise ended.

use bevy::prelude::*;

use super::course::Words;
use crate::home::Base;
use crate::settings::Keymap;
use crate::ship::{Note, Orbital, Reading, Regime};
use crate::space::{Location, Travel};
use crate::ui::units::{distance, duration};

/// What the prompt reads of the cruise.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Said {
    /// What the cruise is doing.
    pub regime: Regime,
    /// Whether a cruise could start now.
    pub ready: bool,
    /// Whether the ship climbs out of the air toward a cruise home.
    pub climbing: bool,
    /// The distances and the time to the target.
    pub reading: Option<Reading>,
    /// Why the last cruise ended, while it is still said.
    pub note: Option<Note>,
    /// Whether the home base stands on the planet below.
    pub home: bool,
}

/// What the cruise puts on the prompt line, each with the key that does it. `taken` is true when
/// the cruise has the line to itself.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Lines {
    /// A sentence with no key: why the cruise did not go on.
    pub block: Option<String>,
    /// The cruise or the glide under way, and the key that stops it.
    pub running: Option<(KeyCode, String)>,
    /// A cruise ready to start, and its key.
    pub ready: Option<(KeyCode, String)>,
    /// Whether the cruise has the line to itself, over the words of the course.
    pub taken: bool,
}

/// Returns `seconds` as a short time to arrival: "45 s", "1 min 20 s", then hours and days.
pub fn clock(seconds: f64) -> String {
    let whole = seconds.max(0.0).round() as u64;
    match whole {
        0..60 => format!("{whole} s"),
        60..3600 => format!("{} min {} s", whole / 60, whole % 60),
        _ => duration(seconds),
    }
}

/// Returns the line of a cruise or a glide under way, as `reading` says.
fn under_way(regime: Regime, reading: Option<Reading>) -> String {
    match (regime, reading) {
        (Regime::Glide, Some(read)) => format!("Gliding down to Home - {}", distance(read.arc)),
        (_, Some(read)) => {
            let glide = read.descent.map_or(String::new(), |left| {
                format!(" - glide in {}", distance(left))
            });
            format!("Home {}, {}{glide}", distance(read.arc), clock(read.eta))
        }
        _ => "Orbital cruise".to_string(),
    }
}

/// Returns the sentence of `note`, with its key when it says what to do next.
fn noted(note: Note, keys: &Keymap) -> Lines {
    let (block, ready) = match note {
        Note::Floor => (
            Some("Cruise ended over the air: the ship slowed first".to_string()),
            None,
        ),
        Note::Near => (Some("Home is close: follow its marker".to_string()), None),
        Note::Arrived => (
            None,
            Some((keys.ship.sink, "Home ahead - land by hand".to_string())),
        ),
    };
    Lines {
        block,
        ready,
        taken: true,
        ..default()
    }
}

/// Returns what the cruise puts on the prompt line from `said`, with the bindings `keys`.
pub fn lines(said: &Said, keys: &Keymap) -> Lines {
    let warp = keys.ship.warp;
    if said.regime != Regime::Off {
        let running = Some((warp, under_way(said.regime, said.reading)));
        return Lines {
            running,
            taken: true,
            ..default()
        };
    }
    if said.climbing {
        let running = Some((warp, "Climbing out of the air to cruise home".to_string()));
        return Lines {
            running,
            taken: true,
            ..default()
        };
    }
    if let Some(note) = said.note {
        return noted(note, keys);
    }
    let start = match said.home {
        true => (keys.map, "Fly home from the map".to_string()),
        false => (warp, "Orbital cruise".to_string()),
    };
    Lines {
        ready: said.ready.then_some(start),
        ..default()
    }
}

/// Returns what the cruise of `orbital` puts on the prompt line for a player `at` a place with
/// the home base where it is, with the bindings `keys`, when a course is `set` or not: nothing
/// while on foot. A climb with no course set is a climb home.
pub fn read(
    orbital: Option<&Orbital>,
    (at, base): (Option<&Location>, Option<&Base>),
    set: bool,
    keys: &Keymap,
) -> Lines {
    let seated = at.is_some_and(|at| at.travel == Travel::InShip);
    let Some(orbital) = orbital.filter(|_| seated) else {
        return Lines::default();
    };
    let said = Said {
        regime: orbital.regime(),
        ready: orbital.ready(),
        climbing: orbital.pending() && !set,
        reading: orbital.reading(),
        note: orbital.note(),
        home: base.zip(at).is_some_and(|(base, at)| base.body == at.body),
    };
    lines(&said, keys)
}

/// The rows of the prompt line the course and the cruise fill: a sentence with no key, a run
/// under way and a start ready, each with its key.
pub type Rows = (
    Option<String>,
    Option<(KeyCode, String)>,
    Option<(KeyCode, String)>,
);

/// Returns the rows of the line from the `cruise` lines and the `course` words, whose key is
/// `engage`: the cruise takes the line while it has it, and a cruise ready to start comes after
/// a course ready to go.
pub fn merge(cruise: Lines, course: Words, engage: KeyCode) -> Rows {
    let keyed = |words: Option<String>| words.map(|words| (engage, words));
    match cruise.taken {
        true => (cruise.block, cruise.running, cruise.ready),
        false => (
            course.block,
            keyed(course.running),
            keyed(course.ready).or(cruise.ready),
        ),
    }
}
