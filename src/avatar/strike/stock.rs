//! Starter stock: what a robot could gather on the planet underfoot, one of each tissue class its
//! life grows and every stone its ground has, for the dev starter tools.

use crate::craft::build::{Intent, Place, Proposal, Stock, solve};
use crate::craft::matter::{Class, Grade, Mineral, Stuff, Treatments, material};
use crate::life::body::Part;
use crate::life::{Life, Species};
use crate::universe::PlanetRecord;
#[cfg(test)]
use crate::world::Globe;

#[cfg(test)]
use super::held::Held;

/// Kilograms of each tissue and stone the starter stock holds.
const PLENTY: f32 = 5.0;

/// Returns the tissue `part` of `species` is made of as a stock, cleanly killed and untreated, or
/// nothing for a tissue too soft to build with.
fn gathered(species: &Species, part: &Part, planet: u64) -> Option<Stock> {
    let tissue = species.tissues.get(usize::from(part.tissue))?;
    let class = Class::of(tissue.kind, part.kind)?;
    let stuff = Stuff::Tissue {
        planet,
        species: species.id.index,
        tissue: part.tissue,
        class,
        grade: Grade::Clean,
        treated: Treatments::default(),
    };
    let made = material(class, tissue, Grade::Clean)?;
    Some(Stock {
        stuff,
        material: made,
        mass: PLENTY,
    })
}

/// Returns the class of a stock of tissue or wood, or nothing for a stone/ammo.
fn class(stock: &Stock) -> Option<Class> {
    match stock.stuff {
        Stuff::Tissue { class, .. } => Some(class),
        Stuff::Wood(_) => Some(Class::Wood),
        _ => None,
    }
}

/// Returns the stock of the planet of terrain seed `planet`: the first tissue of each class among
/// the parts of its species, then each stone that lies on its ground.
pub fn stock(life: &Life, record: &PlanetRecord, planet: u64) -> Vec<Stock> {
    let mut out: Vec<Stock> = Vec::with_capacity(Class::ALL.len() + Mineral::ALL.len());
    let parts = life
        .species
        .iter()
        .flat_map(|s| s.body.parts.iter().map(move |p| (s, p)));
    for found in parts.filter_map(|(species, part)| gathered(species, part, planet)) {
        if !out.iter().any(|s| class(s) == class(&found)) {
            out.push(found);
        }
    }
    let stones = Mineral::ALL.into_iter().filter(|m| m.lies_on(record));
    out.extend(stones.map(|m| Stock {
        stuff: Stuff::Mineral(m),
        material: m.material(),
        mass: PLENTY,
    }));
    out
}

/// Returns the hardest-hitting build the solver makes for `intent` from `stock` at `place`, or
/// nothing when the stock cannot make one.
pub fn hardest(intent: Intent, stock: &[Stock], place: &Place) -> Option<Proposal> {
    Some(solve(intent, stock, place).ok()?.hardest)
}

/// Returns the starter item for `intent` on `globe`: from its life and its stones, on its surface.
#[cfg(test)]
pub fn on(globe: &Globe, intent: Intent) -> Option<Held> {
    let life = globe.life()?;
    let place = Place::on(globe.gravity(), globe.record());
    let stock = stock(life, globe.record(), globe.seed());
    let best = hardest(intent, &stock, &place)?;
    Some(Held::new(best.recipe(&stock), best.build, place))
}
