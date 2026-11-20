//! The card beside the list: the selected entry of the pack, or the workshop's offer, as rows of
//! the field guide's card.

use super::bench::{Bench, intent_name};
use super::lines::{Line, What, treated};
use super::offer::{KINDS, Offer, made_of, title};
use super::sheet::{Sheet, blows, sheet, versus};
use crate::craft::build::{Place, Stock, Why};
use crate::craft::matter::Treatment;
use crate::craft::pack::{Burden, Pack};
use crate::fauna::guide::card::{Card, Row};

/// What the card is drawn from.
pub struct Shown<'a> {
    /// The pack.
    pub pack: &'a Pack,
    /// Its materials as the solver reads them.
    pub stock: &'a [Stock],
    /// Where the robot stands.
    pub place: Place,
    /// What the pack weighs here.
    pub burden: Burden,
    /// The numbers of the held tool, for the workshop.
    pub held: Option<Sheet>,
    /// The treatments this planet allows.
    pub treatments: Vec<Treatment>,
}

/// Returns the rows of the load: mass carried against the capacity, and the pace left.
fn load(burden: &Burden) -> [Row; 3] {
    [
        Row::Heading("Load"),
        Row::Fact(
            "Carried",
            format!("{:.1} of {:.1} kg", burden.load, burden.capacity),
        ),
        Row::Fact("Pace", format!("{:.0} %", burden.pace * 100.0)),
    ]
}

/// Returns the rows of the numbers of a tool.
fn numbers(sheet: &Sheet, held: Option<&Sheet>) -> [Row; 4] {
    [
        Row::Fact("Mass", versus(sheet.mass, held.map(|h| h.mass), " kg", 2)),
        Row::Fact(
            "Swing",
            versus(sheet.speed, held.map(|h| h.speed), " m/s", 1),
        ),
        Row::Fact(
            "Depth",
            versus(sheet.depth, held.map(|h| h.depth), " layers", 2),
        ),
        Row::Fact(
            "Lasts",
            versus(blows(sheet.life), held.map(|h| blows(h.life)), " blows", 0),
        ),
    ]
}

/// Returns the card of the selected row of the pack.
pub fn pack_card(line: Option<&Line>, shown: &Shown) -> Card {
    let Some(line) = line else {
        let mut rows = vec![Row::Note("Nothing carried yet.".to_string())];
        rows.extend(load(&shown.burden));
        return Card {
            title: "Pack".to_string(),
            subtitle: "Harvest a kill or pick up stones".to_string(),
            rows,
        };
    };
    let mut rows = vec![Row::Fact("Mass", format!("{:.2} kg", line.mass))];
    if let What::Item(id) = line.what {
        let item = shown.pack.item(id);
        let build = item.and_then(|item| item.build().map(|b| (item, b)));
        if let Some((item, build)) = build {
            let (worn, joint) = item.worn(&shown.place);
            rows.push(Row::Fact("Wear", format!("{:.0} %", worn * 100.0)));
            rows.push(Row::Note(Why::WeakestJoint(joint).to_string()));
            rows.extend(numbers(&sheet(&build, item.intent(), &shown.place), None));
        }
    }
    rows.extend(load(&shown.burden));
    Card {
        title: line.name.clone(),
        subtitle: line.signature.clone(),
        rows,
    }
}

/// Returns the rows of the treatments the planet allows.
fn here(treatments: &[Treatment]) -> [Row; 2] {
    let names: Vec<&str> = treatments.iter().map(|t| treated(*t)).collect();
    let words = match names.is_empty() {
        true => "none on this world".to_string(),
        false => names.join(", "),
    };
    [Row::Heading("This world"), Row::Fact("Treatments", words)]
}

/// Returns the rows of the three proposals, the chosen one marked.
fn three(bench: &Bench, offer: &Offer, stock: &[Stock]) -> Vec<Row> {
    let mut rows = vec![Row::Heading("Proposals")];
    for (at, proposal) in offer.three.iter().enumerate() {
        let mark = if at == bench.pick { "> " } else { "" };
        let mass = crate::craft::build::mass(&proposal.build.pieces());
        let words = format!("{mark}{} {mass:.2} kg", title(proposal, stock));
        rows.push(Row::Fact(KINDS[at], words));
    }
    rows
}

/// Returns the card of the workshop: the intent, the three proposals, the chosen one against the
/// held tool with its one reason, and the treatments of the planet.
pub fn make_card(bench: &Bench, offer: Result<&Offer, Why>, shown: &Shown) -> Card {
    let mut rows = Vec::with_capacity(16);
    match offer {
        Err(why) => rows.push(Row::Note(why.to_string())),
        Ok(offer) => {
            rows.extend(three(bench, offer, shown.stock));
            rows.push(Row::Heading("Chosen"));
            match &offer.chosen {
                Some(p) => {
                    rows.push(Row::Fact("Made of", made_of(p, shown.stock)));
                    let numbers_of = sheet(&p.build, p.design.intent, &shown.place);
                    rows.extend(numbers(&numbers_of, shown.held.as_ref()));
                    rows.extend(p.why.first().map(|why| Row::Note(why.to_string())));
                }
                None => rows.push(Row::Note("not enough of that material".to_string())),
            }
        }
    }
    rows.extend(here(&shown.treatments));
    rows.extend((!bench.note.is_empty()).then(|| Row::Note(bench.note.clone())));
    Card {
        title: format!("Workshop - {}", intent_name(bench.intent)),
        subtitle: "Numbers in brackets against the held tool".to_string(),
        rows,
    }
}
