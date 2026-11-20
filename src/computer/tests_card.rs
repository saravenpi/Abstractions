//! Computer tests: the card of a planet in the side panel, its subtitle and its facts in
//! their groups, for each class and for a planet not charted.

use super::detail::{Card, Row, card};
use super::tests_detail::{records, world};
use super::*;
use crate::universe::{PlanetRecord, Universe};

/// For each class in the order of `records`: the start of the subtitle, then the air, the
/// water, the temperature, the light, the zone and the tilt.
const CLASSES: [[&str; 7]; 6] = [
    [
        "Ocean planet",
        "like home, pressure 1.20",
        "liquid, oceans 95 %",
        "22 C",
        "1.00 x home",
        "in the habitable zone",
        "11 deg",
    ],
    [
        "Island planet",
        "like home, pressure 1.00",
        "liquid, oceans 60 %",
        "14 C",
        "1.00 x home",
        "in the habitable zone",
        "23 deg",
    ],
    [
        "Desert planet",
        "thin, pressure 0.40",
        "none",
        "45 C",
        "1.80 x home",
        "in the habitable zone",
        "6 deg",
    ],
    [
        "Ice planet",
        "thin, pressure 0.20",
        "ice",
        "-120 C",
        "0.04 x home",
        "too cold for water",
        "57 deg",
    ],
    [
        "Lava planet",
        "thick, pressure 92.0",
        "steam, molten rock 30 %",
        "1200 C",
        "1500 x home",
        "too hot for water",
        "0 deg",
    ],
    [
        "Barren planet",
        "none, pressure 0.00",
        "none",
        "-273 C",
        "0.00 x home",
        "too cold for water",
        "180 deg",
    ],
];

/// The labels of the rows of the card of a planet with neither moons nor rings, in order.
const ORDER: [&str; 14] = [
    "Surface",
    "Air",
    "Water",
    "Climate",
    "Temperature",
    "Light",
    "Zone",
    "Tilt",
    "Body",
    "Radius",
    "Gravity",
    "Time",
    "Day",
    "Year",
];

/// Returns the value of the fact labelled `label` on `card`, if it has one.
pub(super) fn fact<'a>(card: &'a Card, label: &str) -> Option<&'a str> {
    card.rows.iter().find_map(|row| match row {
        Row::Fact(name, value) if *name == label => Some(value.as_str()),
        _ => None,
    })
}

/// The card of each class names its kind and its star, and says the whole record in
/// labelled facts with plain words and units.
#[test]
fn the_card_says_the_whole_record_of_each_class() {
    let universe = Universe::new(7);
    let labels = ["Air", "Water", "Temperature", "Light", "Zone", "Tilt"];
    for (record, wanted) in records().into_iter().zip(CLASSES) {
        let (line, planet) = world(record);
        let card = card(&line, Some(&planet), &universe);
        let star = Target::Star(planet.id.star).name(&universe);
        assert_eq!(card.title, planet.name);
        assert_eq!(card.subtitle, format!("{} of {star}", wanted[0]));
        let said = labels.map(|label| fact(&card, label).unwrap_or_default());
        assert_eq!(said, wanted[1..], "{}", wanted[0]);
    }
}

/// The card of a planet goes through its surface, climate, body and time, one fact per row,
/// with no distance, which its row of the list shows, and a planet not charted gives its size
/// and what to do to learn more.
#[test]
fn the_card_groups_the_facts_in_order() {
    let universe = Universe::new(7);
    let (line, planet) = world(PlanetRecord::TEMPERATE);
    let home = card(&line, Some(&planet), &universe);
    let order: Vec<&str> = home
        .rows
        .iter()
        .map(|row| match row {
            Row::Heading(name) | Row::Fact(name, _) | Row::Note(name) => *name,
        })
        .collect();
    assert_eq!(order, ORDER);
    let facts = ["Radius", "Gravity", "Day", "Year"].map(|label| fact(&home, label));
    let shown = ["6.00 km", "1.00 g", "24 min", "3 h 20 min"].map(Some);
    assert_eq!(facts, shown);
    assert_eq!(fact(&home, "From you"), None);
    let uncharted = card(&line, None, &universe);
    assert!(uncharted.subtitle.starts_with("Uncharted planet of "));
    let advice = Row::Note("Visit this system to chart its planets");
    assert_eq!(uncharted.rows.last(), Some(&advice));
}
