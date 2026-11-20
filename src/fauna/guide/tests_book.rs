//! Tests of the list and the card: the list holds exactly the species of the planet's web, in
//! guide order, unseen ones unknown, and a dead planet says it has no life.

use super::book::{LIFELESS, UNKNOWN, book};
use super::card::{Row, card};
use super::rig::web;
use super::seen::Seen;
use crate::life::Guild;

/// Every species of the web shows once, plants first by growth architecture, and unseen ones
/// are unknown until marked.
#[test]
fn the_list_matches_the_planet_web() {
    let life = web();
    let mut seen = Seen::default();
    let list = book(Some(&life), &seen);
    assert_eq!(list.entries.len(), life.species.len());
    let mut indices: Vec<u16> = list.entries.iter().map(|e| e.index).collect();
    indices.sort_unstable();
    assert_eq!(indices, (0..life.species.len() as u16).collect::<Vec<_>>());
    let plants = life
        .species
        .iter()
        .filter(|s| s.role.guild == Guild::Producer)
        .count();
    let growth: Vec<u8> = list.entries[..plants]
        .iter()
        .map(|e| life.species[usize::from(e.index)].archetype)
        .collect();
    assert!(growth.windows(2).all(|w| w[0] <= w[1]), "{growth:?}");
    assert!(list.entries.iter().all(|e| e.name == UNKNOWN && !e.seen));
    assert_eq!(list.tally(), format!("0 of {} seen", life.species.len()));
    let first = &life.species[usize::from(list.entries[plants].index)];
    assert!(seen.mark(first.id) && !seen.mark(first.id));
    let list = book(Some(&life), &seen);
    assert_eq!(list.entries[plants].name, first.name);
    assert_eq!(list.tally(), format!("1 of {} seen", life.species.len()));
}

/// A seen animal's card groups its facts, and names only the seen species it eats.
#[test]
fn the_card_groups_facts_and_hides_the_unseen() {
    let life = web();
    let eater = life
        .species
        .iter()
        .find(|s| !s.prey.is_empty())
        .expect("an eater");
    let mut seen = Seen::default();
    assert_eq!(
        card(Some(&life), Some(eater.id.index), &seen).title,
        UNKNOWN
    );
    seen.mark(eater.id);
    let shown = card(Some(&life), Some(eater.id.index), &seen);
    assert_eq!(shown.title, eater.name);
    let heads: Vec<&str> = shown
        .rows
        .iter()
        .filter_map(|r| match r {
            Row::Heading(h) => Some(*h),
            _ => None,
        })
        .collect();
    assert_eq!(heads, ["Body", "Life", "Habitat", "Relations"]);
    let eats = shown.rows.iter().find_map(|r| match r {
        Row::Fact("Eats", v) => Some(v.clone()),
        _ => None,
    });
    assert_eq!(eats, Some(format!("{} unknown", eater.prey.len())));
}

/// A dead planet lists nothing and says so.
#[test]
fn a_dead_planet_shows_no_life() {
    let list = book(None, &Seen::default());
    assert!(list.entries.is_empty());
    assert_eq!(list.tally(), LIFELESS);
    assert_eq!(
        card(None, None, &Seen::default()).title,
        "No life found on this planet"
    );
}
