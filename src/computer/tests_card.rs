//! Computer tests: the card of a planet in the side panel, its subtitle and its stats in their
//! groups, for each class and for a planet not charted.

use super::detail::{Card, Row, card, visible};
use super::tests_detail::{records, world};
use super::*;
use crate::universe::{PlanetRecord, PlanetSpec, Universe};

/// For each class in the order of `records`: the start of the subtitle, then the value and the
/// note of the air, the water, the value and the note of the temperature, the light, the zone
/// and the tilt.
const CLASSES: [[&str; 9]; 6] = [
    [
        "Ocean planet",
        "1.2 x home",
        "1.2 bar",
        "liquid, oceans 95 %",
        "22 C",
        "Mild",
        "1.00 x home",
        "in the habitable zone",
        "11 deg",
    ],
    [
        "Island planet",
        "1 x home",
        "1 bar",
        "liquid, oceans 60 %",
        "14 C",
        "Mild",
        "1.00 x home",
        "in the habitable zone",
        "23 deg",
    ],
    [
        "Desert planet",
        "0.4 x home",
        "0.4 bar",
        "none",
        "45 C",
        "Hot",
        "1.80 x home",
        "in the habitable zone",
        "6 deg",
    ],
    [
        "Ice planet",
        "0.2 x home",
        "0.2 bar",
        "ice",
        "-120 C",
        "Freezing",
        "0.04 x home",
        "too cold for water",
        "57 deg",
    ],
    [
        "Lava planet",
        "92 x home",
        "92 bar",
        "steam, molten rock 30 %",
        "1200 C",
        "Scorching",
        "1500 x home",
        "too hot for water",
        "0 deg",
    ],
    [
        "Barren planet",
        "0 x home",
        "0 bar",
        "none",
        "-273 C",
        "Freezing",
        "0.00 x home",
        "too cold for water",
        "180 deg",
    ],
];

/// What the rows of the card of a planet with neither moons nor rings are, in order: the
/// headings and the labels, a meaning as `~` and the More row as `+`.
const ORDER: [&str; 18] = [
    "Size and gravity",
    "Radius",
    "Gravity",
    "~",
    "Air and heat",
    "Air",
    "Temperature",
    "~",
    "Day and year",
    "Day",
    "Year",
    "~",
    "Water and life",
    "Water",
    "+",
    "Light",
    "Zone",
    "Tilt",
];

/// Returns the value of the fact labelled `label` on `card`, if it has one.
pub(super) fn fact<'a>(card: &'a Card, label: &str) -> Option<&'a str> {
    stat(card, label).map(|(value, _)| value)
}

/// Returns the value and the note of the stat labelled `label` on `card`, if it has one.
pub(super) fn stat<'a>(card: &'a Card, label: &str) -> Option<(&'a str, &'a str)> {
    card.rows.iter().find_map(|row| match row {
        Row::Stat {
            label: name,
            value,
            note,
            ..
        } if *name == label => Some((value.as_str(), note.as_str())),
        _ => None,
    })
}

/// The card of each class names its kind and its star, and says the record in stats with units
/// and plain words, and the rarer facts behind More.
#[test]
fn the_card_says_the_whole_record_of_each_class() {
    let universe = Universe::new(7);
    for (record, wanted) in records().into_iter().zip(CLASSES) {
        let (line, planet) = world(record);
        let card = card(&line, Some(&planet), &universe);
        let star = Target::Star(planet.id.star).name(&universe);
        assert_eq!(card.title, planet.name);
        assert_eq!(card.subtitle, format!("{} of {star}", wanted[0]));
        let air = stat(&card, "Air").unwrap_or_default();
        let heat = stat(&card, "Temperature").unwrap_or_default();
        let water = stat(&card, "Water").unwrap_or_default().0;
        let more = ["Light", "Zone", "Tilt"].map(|label| fact(&card, label).unwrap_or_default());
        let said = [
            air.0, air.1, water, heat.0, heat.1, more[0], more[1], more[2],
        ];
        assert_eq!(said, wanted[1..], "{}", wanted[0]);
    }
}

/// The card of a planet goes through size and gravity, air and heat, day and year, then water
/// and life, with the rarer facts behind More. Nothing says how far the place is, which its row
/// of the list shows. A planet not charted gives its size and what to do to learn more.
#[test]
fn the_card_groups_the_stats_in_order() {
    let universe = Universe::new(7);
    let (line, planet) = world(PlanetRecord::TEMPERATE);
    let planet = PlanetSpec { moons: 0, ..planet };
    let home = card(&line, Some(&planet), &universe);
    let name = |row: &Row| match row {
        Row::Heading(_, name) | Row::Note(name) => name.to_string(),
        Row::Stat { label, .. } => label.to_string(),
        Row::Meaning(_) => "~".to_string(),
        Row::More => "+".to_string(),
    };
    let order: Vec<String> = home.rows.iter().map(name).collect();
    assert_eq!(order, ORDER);
    let stats = ["Radius", "Gravity", "Day", "Year"].map(|label| stat(&home, label));
    let value = |at: usize| stats[at].map(|(value, _)| value);
    let shown = ["6 km", "1 x home", "24 min", "3 h 20 min"];
    assert_eq!([0, 1, 2, 3].map(value), shown.map(Some));
    let versus = [0, 2, 3].map(|at| stats[at].unwrap().1);
    assert!(versus.iter().all(|note| note.ends_with(" x home")));
    assert_eq!(fact(&home, "From you"), None);
    let folded = visible(&home.rows, false);
    assert_eq!(folded.last(), Some(&Row::More));
    assert_eq!(visible(&home.rows, true).len(), home.rows.len());
    let uncharted = card(&line, None, &universe);
    assert!(uncharted.subtitle.starts_with("Uncharted planet of "));
    let advice = Row::Note("Not visited. Fly there to learn its air, water and climate.");
    assert_eq!(uncharted.rows.last(), Some(&advice));
    assert_eq!(uncharted.rows.len(), 5);
}

/// Against itself, the home planet reads "1.0 x home" for its radius, its day and its year, "1 x home"
/// for its gravity and its air, and a stat always writes its unit.
#[test]
fn the_home_planet_reads_one_against_itself() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let (line, _) = world(PlanetRecord::TEMPERATE);
    let shown = card(&line, Some(&home), &universe);
    for label in ["Radius", "Day", "Year"] {
        let (_, note) = stat(&shown, label).unwrap();
        assert_eq!(note, "1.0 x home", "{label}");
    }
    let units = [
        ("Radius", " km"),
        ("Gravity", " x home"),
        ("Air", " x home"),
    ];
    for (label, unit) in units {
        let (value, _) = stat(&shown, label).unwrap();
        assert!(value.ends_with(unit), "{label}: {value}");
    }
    let (heat, _) = stat(&shown, "Temperature").unwrap();
    assert!(heat.ends_with(" C"), "{heat}");
}
