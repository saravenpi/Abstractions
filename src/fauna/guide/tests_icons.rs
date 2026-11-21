//! Tests of the pictures and the plain words of the guide: every row of every card of real
//! planets has an icon, every row of the list has a silhouette, a species not seen yet wears a
//! question mark and explains the guide, and the temper and the yield of each species are said.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::book::{UNKNOWN, book};
use super::card::{Row, WHAT, card, how};
use super::icons::{GROWTHS, LABELS, of_label, sign};
use super::open::{Shelf, wheel};
use super::rig::lives;
use super::seen::Seen;
use super::temper::{BOLD, HUNTS, SHY, temper, yields};
use super::words::GROWTHS as GROWTH_NAMES;
use crate::life::{Guild, Life};
use crate::ui::{Icon, Scroller, Window};
use bevy::ecs::system::RunSystemOnce;

/// Returns `seen` with every species of `life` marked.
fn all(life: &Life) -> Seen {
    let mut seen = Seen::default();
    for species in &life.species {
        seen.mark(species.id);
    }
    seen
}

/// Returns the label of `row`, a heading or a fact, and nothing for a sentence.
fn label(row: &Row) -> Option<&'static str> {
    match row {
        Row::Heading(label) | Row::Fact(label, _) => Some(*label),
        Row::Note(_) => None,
    }
}

/// Every heading and every fact of every card of the living home planets of 40 universes has an
/// icon, and no label of the table is written twice.
#[test]
fn every_row_of_every_card_has_an_icon() {
    let mut rows = 0;
    for life in lives(40) {
        let seen = all(&life);
        for species in &life.species {
            let shown = card(Some(&life), Some(species.id.index), &seen);
            for label in shown.rows.iter().filter_map(label) {
                assert!(of_label(label).is_some(), "no icon for {label}");
                rows += 1;
            }
        }
    }
    assert!(rows > 500, "{rows} rows");
    for (at, (label, _)) in LABELS.iter().enumerate() {
        assert!(
            LABELS[..at].iter().all(|(other, _)| other != label),
            "{label}"
        );
    }
    assert_eq!(GROWTHS.len(), GROWTH_NAMES.len());
}

/// A seen species shows its silhouette in the list and a plant or an animal on its card; one not
/// seen yet shows a question mark in both, is called unknown, and its card says how to see it
/// and what the guide is.
#[test]
fn seen_and_unseen_species_look_different() {
    let life = lives(8).into_iter().next().expect("a living home planet");
    let none = Seen::default();
    for entry in book(Some(&life), &none).entries {
        assert_eq!((entry.icon, entry.name.as_str()), (Icon::Unknown, UNKNOWN));
        assert_eq!(sign(Some(&life), Some(entry.index), &none), Icon::Unknown);
        let shown = card(Some(&life), Some(entry.index), &none);
        let notes = [Row::Note(how()), Row::Note(WHAT.to_string())];
        assert_eq!(shown.rows, notes);
    }
    let seen = all(&life);
    for entry in book(Some(&life), &seen).entries {
        assert_ne!(entry.icon, Icon::Unknown, "{}", entry.name);
        let species = &life.species[usize::from(entry.index)];
        let plant = species.role.guild == Guild::Producer;
        let wanted = [Icon::Animal, Icon::Plant][usize::from(plant)];
        assert_eq!(sign(Some(&life), Some(entry.index), &seen), wanted);
    }
    assert_eq!(sign(None, None, &none), Icon::Guide);
    assert!(WHAT.contains("field guide") && how().contains(" m "));
}

/// Every animal says what it does near a robot in one of three ways, a plant says nothing, and
/// every species says what it yields.
#[test]
fn every_species_says_its_temper_and_its_yield() {
    let mut tempers = std::collections::HashSet::new();
    for life in lives(40) {
        for species in &life.species {
            let plant = species.role.guild == Guild::Producer;
            let mood = temper(species);
            assert_eq!(mood.is_none(), plant, "{}", species.name);
            assert!(mood.is_none_or(|words| [SHY, BOLD, HUNTS].contains(&words)));
            tempers.extend(mood);
            assert!(!yields(species).is_empty(), "{}", species.name);
        }
    }
    assert!(tempers.contains(SHY) && tempers.len() >= 2, "{tempers:?}");
}

/// The wheel scrolls the list, and leaves it alone while the pointer is over the facts of the
/// card, which then scroll instead.
#[test]
fn the_wheel_scrolls_the_list_only_away_from_the_card() {
    let life = lives(8).into_iter().next().expect("a living home planet");
    let mut app = App::new();
    app.insert_resource(Shelf {
        book: book(Some(&life), &Seen::default()),
        rows: 2,
        window: Window::default(),
        ..default()
    });
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, -1.0),
    });
    let over = |cursor_over: bool| RelativeCursorPosition {
        cursor_over,
        normalized: None,
    };
    let facts = app.world_mut().spawn((Scroller, over(true))).id();
    app.world_mut().run_system_once(wheel).unwrap();
    assert_eq!(app.world().resource::<Shelf>().window.top, 0);
    app.world_mut().entity_mut(facts).insert(over(false));
    app.world_mut().run_system_once(wheel).unwrap();
    assert_eq!(app.world().resource::<Shelf>().window.top, 1);
}
