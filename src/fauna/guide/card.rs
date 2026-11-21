//! The card of the selected species: its name, what it is, and its facts in groups, as the map
//! card shows a place: Body, Life, Habitat and Relations.

use super::book::{LIFELESS, UNKNOWN};
use super::relations;
use super::seen::{SIGHT, Seen};
use super::temper::{temper, yields};
use super::words::{active, climate, diet, habitat, kind, limbs, moves, plan};
use crate::life::{Guild, Life, Species};
use crate::ui::units::fixed;

/// One row of a card.
#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    /// The name of a group, in spaced capitals.
    Heading(&'static str),
    /// A muted label and its value.
    Fact(&'static str, String),
    /// A line of plain words.
    Note(String),
}

/// What the card shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Card {
    /// The name.
    pub title: String,
    /// What it is.
    pub subtitle: String,
    /// Its facts.
    pub rows: Vec<Row>,
}

/// Returns a weight in kilograms in words: grams under 1 kg, tonnes from 1000 kg.
pub fn weight(kg: f32) -> String {
    crate::ui::units::mass(f64::from(kg))
}

/// Returns the Body rows of an animal: its size, weight, plan, limbs and gait, and what its body
/// yields when harvested.
fn body(species: &Species) -> Vec<Row> {
    let body = &species.body;
    vec![
        Row::Heading("Body"),
        Row::Fact(
            "Size",
            format!(
                "{} m long, {} m tall",
                fixed(body.length, 1),
                fixed(body.height, 1)
            ),
        ),
        Row::Fact("Weight", weight(species.history.mass)),
        Row::Fact("Plan", plan(species.plan).to_string()),
        Row::Fact("Limbs", limbs(body)),
        Row::Fact("Moves", moves(body.locomotion).to_string()),
        Row::Fact("Yields", yields(species)),
    ]
}

/// Returns the Life rows of an animal: its diet, herd, hours and young, and what it does when
/// a robot comes near.
fn living(species: &Species, life: &Life) -> Vec<Row> {
    let history = &species.history;
    let herd = match (history.group_min, history.group_max) {
        (_, 0 | 1) => "Lives alone".to_string(),
        (low, high) if low == high => format!("{high} together"),
        (low, high) => format!("{} to {high}", low.max(1)),
    };
    let guard = if history.care > 0.5 { ", guarded" } else { "" };
    let mood = temper(species).map(|words| Row::Fact("Temper", words.to_string()));
    let rows = [
        Row::Heading("Life"),
        Row::Fact(
            "Diet",
            diet(species.role.guild, life.chemistry.energy).to_string(),
        ),
        Row::Fact("Herd", herd),
        Row::Fact("Active", active(species.role.activity).to_string()),
        Row::Fact(
            "Young",
            format!("{} per brood{guard}", history.clutch.max(1)),
        ),
    ];
    rows.into_iter().chain(mood).collect()
}

/// Returns the Habitat rows: where it lives and the climate it stands.
fn home(species: &Species) -> Vec<Row> {
    let (temperature, moisture) = climate(&species.tolerance);
    vec![
        Row::Heading("Habitat"),
        Row::Fact("Lives", habitat(species.role.habitat).to_string()),
        Row::Fact("Climate", temperature),
        Row::Fact("Moisture", moisture),
    ]
}

/// Returns the card of a seen species.
fn known(life: &Life, species: &Species, seen: &Seen) -> Card {
    let what = kind(species.role.guild, species.archetype);
    let mut rows = match species.role.guild {
        Guild::Producer => vec![
            Row::Heading("Body"),
            Row::Fact("Growth", what.to_string()),
            Row::Fact("Yields", yields(species)),
            Row::Heading("Life"),
            Row::Fact(
                "Food",
                diet(Guild::Producer, life.chemistry.energy).to_string(),
            ),
        ],
        _ => [body(species), living(species, life)].concat(),
    };
    rows.extend(home(species));
    let ties = relations::rows(life, seen, species.id.index);
    if !ties.is_empty() {
        rows.push(Row::Heading("Relations"));
        rows.extend(ties);
    }
    let class = if species.role.guild == Guild::Producer {
        "Plant"
    } else {
        "Animal"
    };
    Card {
        title: species.name.clone(),
        subtitle: format!("{class}, {}", species.epithet),
        rows,
    }
}

/// What the guide is and how a species gets into it, in one sentence, for a card with nothing
/// else to say.
pub const WHAT: &str = "The field guide lists every species of the planet you stand on: its \
plants once you land, and an animal once you have come close to one.";

/// Returns how to learn about a species not seen yet.
pub fn how() -> String {
    format!(
        "Come within {} m of one to learn about it.",
        fixed(SIGHT, 0)
    )
}

/// Returns the card of species `index` of `life`, an unknown card while it is unseen, and the
/// lifeless card without life.
pub fn card(life: Option<&Life>, index: Option<u16>, seen: &Seen) -> Card {
    let Some(life) = life.filter(|l| !l.species.is_empty()) else {
        return Card {
            title: LIFELESS.to_string(),
            subtitle: "The guide stays empty here".to_string(),
            rows: vec![Row::Note(WHAT.to_string())],
        };
    };
    let species = index.and_then(|i| life.species.get(usize::from(i)));
    match species {
        Some(s) if seen.has(s.id) => known(life, s, seen),
        _ => Card {
            title: UNKNOWN.to_string(),
            subtitle: "Not seen yet".to_string(),
            rows: vec![Row::Note(how()), Row::Note(WHAT.to_string())],
        },
    }
}
