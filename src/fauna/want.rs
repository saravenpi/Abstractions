//! Which animals should be drawn: herd members and carcass visitors near the local robot, nearest
//! first.

use bevy::math::DVec3;

use super::herds::{herd_arc, member};
use super::tune::{REACH, SLACK, VISIT_DELAY, VISITORS};
use crate::life::herd::{Herd, Ledger};
use crate::life::hunt::Carcass;
use crate::life::{Guild, Life, Species};
use crate::universe::hash::{Rng, child};

/// What leads an animal: its place in a herd, or the carcass it visits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lead {
    /// Member `slot` of herd `id`.
    Herd {
        /// Identity of the herd.
        id: u64,
        /// Slot in the herd.
        slot: u8,
    },
    /// The carcass of identity `key`, lying at unit direction `at`.
    Carcass {
        /// Identity of the carcass.
        key: u64,
        /// Unit direction where it lies.
        at: DVec3,
    },
    /// Visitor `slot` of the carcass of identity `key`.
    Visit {
        /// Identity of the carcass.
        key: u64,
        /// Number of the visitor.
        slot: u8,
        /// Unit direction of the carcass.
        at: DVec3,
    },
}

impl Lead {
    /// Returns the identity of the group and the slot in it.
    pub fn key(&self) -> (u64, u8) {
        match *self {
            Lead::Herd { id, slot } => (id, slot),
            Lead::Carcass { key, .. } => (key, u8::MAX),
            Lead::Visit { key, slot, .. } => (key, slot),
        }
    }
}

/// An animal that should be drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Want {
    /// What leads it.
    pub lead: Lead,
    /// Index of its species.
    pub species: u16,
    /// Unit direction of its place now.
    pub dir: DVec3,
    /// Metres along the ground from the local robot.
    pub metres: f64,
}

/// Returns true when animals of `species` have a body to draw.
pub fn drawn(species: &Species) -> bool {
    species.role.guild != Guild::Producer && !species.body.parts.is_empty()
}

/// Returns the identity of `carcass`.
pub fn carcass_key(carcass: &Carcass) -> u64 {
    child(carcass.herd ^ 0xCA5C, u64::from(carcass.slot)) ^ carcass.since.to_bits()
}

/// The robot and the time wants are read for.
#[derive(Debug, Clone, Copy)]
pub struct From {
    /// Unit direction of the local robot.
    pub robot: DVec3,
    /// Universe time in seconds.
    pub time: f64,
    /// Sea level radius of the planet in metres.
    pub radius: f64,
}

/// Fills `out` with every member of `herds` within [`REACH`] and [`SLACK`] of the robot, nearest
/// first, leaving out those dead by `ledger`.
pub fn members(life: &Life, (herds, ledger): (&[Herd], &Ledger), from: From, out: &mut Vec<Want>) {
    out.clear();
    for herd in herds {
        let drawable = life
            .species
            .get(usize::from(herd.species))
            .is_some_and(drawn);
        let alive = |slot: &u8| drawable && !ledger.dead(herd.id, *slot, from.time);
        for slot in (0..herd.base).filter(alive) {
            let Some((_, dir)) = member(herd, slot, from.time, from.radius) else {
                continue;
            };
            let lead = Lead::Herd { id: herd.id, slot };
            push(out, (lead, herd.species, dir), from);
        }
    }
}

/// Adds to `out` the carcasses that still have meat and the visitors that have come to them by then,
/// within [`REACH`] and [`SLACK`] of the robot, then sorts `out` nearest first.
pub fn visitors(life: &Life, carcasses: &[Carcass], from: From, out: &mut Vec<Want>) {
    for c in carcasses.iter().filter(|c| from.time < c.meat) {
        let key = carcass_key(c);
        push(
            out,
            (Lead::Carcass { key, at: c.at }, c.species, c.at),
            from,
        );
        for slot in 0..VISITORS {
            let come = c.since + VISIT_DELAY * f64::from(slot + 1);
            let lead = Lead::Visit {
                key,
                slot,
                at: c.at,
            };
            if let Some(species) = visitor(life, c, (key, slot)).filter(|_| from.time >= come) {
                push(out, (lead, species, c.at), from);
            }
        }
    }
    out.sort_unstable_by(|a, b| {
        a.metres
            .total_cmp(&b.metres)
            .then(a.lead.key().cmp(&b.lead.key()))
    });
}

/// Returns the species index of visitor `slot` of the carcass `c` of identity `key`: one of its
/// visitor species with a body to draw, by a seeded roll.
fn visitor(life: &Life, c: &Carcass, (key, slot): (u64, u8)) -> Option<u16> {
    let kinds = || {
        c.visitors
            .iter()
            .filter(|&&s| life.species.get(usize::from(s)).is_some_and(drawn))
    };
    let count = kinds().count() as u32;
    let pick = Rng::new(child(key, u64::from(slot))).below(count.max(1));
    kinds().nth(pick as usize).copied()
}

/// Adds the animal led by `lead`, of species index `species`, at unit direction `dir` to `out` when it
/// is within [`REACH`] and [`SLACK`] of the robot.
fn push(out: &mut Vec<Want>, (lead, species, dir): (Lead, u16, DVec3), from: From) {
    let metres = herd_arc(dir, from.robot) * from.radius;
    if metres <= REACH + SLACK {
        out.push(Want {
            lead,
            species,
            dir,
            metres,
        });
    }
}

/// Returns true when a drawn animal led by `lead`, standing at unit direction `here`, stays drawn though
/// it is not wanted: a herd member living by `ledger` within [`REACH`] and [`SLACK`] of the robot of
/// `from`, wherever its place in its herd went.
pub fn lingers(lead: Lead, here: DVec3, from: From, ledger: &Ledger) -> bool {
    let near = herd_arc(here, from.robot) * from.radius <= REACH + SLACK;
    match lead {
        Lead::Herd { id, slot } => near && !ledger.dead(id, slot, from.time),
        _ => false,
    }
}
