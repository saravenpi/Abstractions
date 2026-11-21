//! The kit a robot wakes with: a spear and a knife knapped from the stone underfoot, the lightest the
//! workshop proposes, so the first animal that goes for the robot can be fought, and every tool made
//! after from bone, horn and sinew is better. Each tool is a real build with its mass and its wear.
//!
//! A pack is given its kit once. A new world's pack is not kitted, so it gets one. A world saved
//! before kits were given reads `kitted` as false, so its pack gets the kit too, but only when it
//! holds no tool, in the pack or out in the world; either way it is kitted from then on. In a session
//! every player has a pack of their own and gets a kit of their own.

use bevy::prelude::*;

use super::tune::KIT_STONE;
use super::{Pack, SLOTS};
use crate::craft::build::{Intent, Place, Proposal, Stock, solve};
use crate::craft::matter::{Material, Mineral, Stuff};
use crate::universe::PlanetRecord;
use crate::world::Terrain;

/// The tools of the kit, in the order of the hotbar, each with the tool made in its place when the
/// stone underfoot takes no edge: a spear, else a club, then a knife.
const KIT: [(Intent, Option<Intent>); 2] =
    [(Intent::Throw, Some(Intent::Crush)), (Intent::Cut, None)];

/// Returns the stones that lie on the ground of a planet of `record`, [`KIT_STONE`] kilograms of
/// each, as the workshop reads them.
fn stones(record: &PlanetRecord) -> Vec<Stock> {
    let lying = Mineral::ALL.into_iter().filter(|m| m.lies_on(record));
    lying
        .map(|m| Stock {
            stuff: Stuff::Mineral(m),
            material: m.material(),
            mass: KIT_STONE,
        })
        .collect()
}

/// Returns the lightest build the workshop proposes for `intent`, else for `instead`, from `stock`
/// at `place`.
fn lightest(
    intent: Intent,
    instead: Option<Intent>,
    stock: &[Stock],
    place: &Place,
) -> Option<Proposal> {
    let made = |intent: Intent| solve(intent, stock, place).ok().map(|p| p.lightest);
    made(intent).or_else(|| instead.and_then(made))
}

impl Pack {
    /// Returns true when the pack holds a tool, in the pack or out in the world.
    pub fn armed(&self) -> bool {
        !self.items.is_empty() || !self.out.is_empty()
    }

    /// Keeps the tool of `proposal` made from `stock` without taking anything from the pack, and
    /// returns its id, or nothing when a material of its recipe is not in `stock`.
    fn knap(&mut self, proposal: &Proposal, stock: &[Stock]) -> Option<u32> {
        let recipe = proposal.recipe(stock);
        let material = |stuff: &Stuff| stock.iter().find(|s| s.stuff == *stuff).map(|s| s.material);
        let materials: Option<Vec<Material>> = recipe.stuff.iter().map(material).collect();
        Some(self.keep(recipe, materials?))
    }

    /// Gives the pack its kit on a planet of `record` under `gravity` metres per second squared,
    /// once: unless the pack already holds a tool, it makes each tool of [`KIT`] that the stone
    /// underfoot allows, in the first hotbar slots, and holds the first empty slot, so the robot
    /// wakes empty-handed and draws the spear with the key of slot 1. The pack is kitted afterwards either way. Returns
    /// how many tools it made.
    pub fn equip(&mut self, record: &PlanetRecord, gravity: f64) -> usize {
        if self.kitted {
            return 0;
        }
        self.kitted = true;
        if self.armed() {
            return 0;
        }
        let (stock, place) = (stones(record), Place::on(gravity, record));
        let made = KIT
            .into_iter()
            .filter_map(|(intent, instead)| lightest(intent, instead, &stock, &place))
            .filter_map(|proposal| self.knap(&proposal, &stock))
            .count();
        let empty = self.bar.slots.iter().position(Option::is_none);
        self.bar.select(empty.unwrap_or(SLOTS));
        made
    }
}

/// Run condition: the pack has not been given its kit.
pub(crate) fn unkitted(pack: Res<Pack>) -> bool {
    !pack.kitted
}

/// Gives the pack of the local robot its kit from the stone of the planet in view.
pub(crate) fn kit(terrain: Option<Res<Terrain>>, mut pack: ResMut<Pack>) {
    let Some(terrain) = terrain else {
        return;
    };
    let globe = terrain.globe();
    let made = pack.equip(globe.record(), globe.gravity());
    info!("kit: {made} stone tools in the pack");
}
