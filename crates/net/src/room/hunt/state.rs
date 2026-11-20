//! How the wounds and kills of the hunt are recorded, each checked against what the room
//! already holds.

use super::{BY_MAX, HURT_KEEP, HURT_MAX, Hunting, Hurt};
use crate::log::PlayerId;
use crate::wire::{AnimalKey, CREDIT_WINDOW, Carcass, Kill, LASTS, SKEW};

impl Hunting {
    /// Returns true when `animal` lies dead.
    pub(in crate::room) fn dead(&self, animal: &AnimalKey) -> bool {
        self.carcasses.iter().any(|c| c.animal == *animal)
    }

    /// Returns the record of `animal`, started at `now` when there is none and the room has
    /// room for it.
    fn wound(&mut self, animal: AnimalKey, now: f64) -> Option<&mut Hurt> {
        if self.hurt.len() >= HURT_MAX && !self.hurt.contains_key(&animal) {
            self.hurt.retain(|_, hurt| now - hurt.last < HURT_KEEP);
        }
        if self.hurt.len() >= HURT_MAX && !self.hurt.contains_key(&animal) {
            return None;
        }
        let fresh = || Hurt {
            broken: 0,
            last: now,
            by: Vec::new(),
        };
        Some(self.hurt.entry(animal).or_insert_with(fresh))
    }

    /// Notes that `by` hurt `animal` at universe time `now`.
    pub(in crate::room) fn strike(&mut self, animal: AnimalKey, by: PlayerId, now: f64) {
        let Some(hurt) = self.wound(animal, now) else {
            return;
        };
        hurt.last = now;
        hurt.by.retain(|(who, _)| *who != by);
        hurt.by.push((by, now));
        if hurt.by.len() > BY_MAX {
            hurt.by.remove(0);
        }
    }

    /// Breaks `part` of `animal` and returns true when it was not broken before.
    pub(in crate::room) fn crack(&mut self, animal: AnimalKey, part: u8, now: f64) -> bool {
        let Some(hurt) = self.wound(animal, now) else {
            return false;
        };
        let was = hurt.broken;
        hurt.broken |= 1 << part;
        hurt.broken != was
    }

    /// Records the kill `kill` by `by`, seen at universe time `now`, and returns its carcass.
    /// A kill of an animal that already lies dead is refused, and a time far from `now` is
    /// replaced by it.
    pub(in crate::room) fn kill(&mut self, kill: Kill, by: PlayerId, now: f64) -> Option<Carcass> {
        if self.dead(&kill.animal) {
            return None;
        }
        let hurt = self.hurt.remove(&kill.animal);
        self.wounds.retain(|(animal, _)| *animal != kill.animal);
        let credited = hurt.iter().flat_map(|hurt| &hurt.by);
        let recent = credited.filter(|(_, at)| now - at <= CREDIT_WINDOW);
        let mut credited: Vec<PlayerId> = recent.map(|(who, _)| *who).collect();
        if !credited.contains(&by) {
            credited.push(by);
        }
        let since = Some(kill.since).filter(|at| (at - now).abs() <= SKEW);
        let carcass = Carcass {
            animal: kill.animal,
            species: kill.species,
            at: kill.at,
            since: since.unwrap_or(now),
            grade: kill.grade.min(2),
            lasts: kill.lasts.clamp(LASTS.0, LASTS.1),
            taken: 0,
            broken: hurt.map_or(0, |hurt| hurt.broken),
            credited,
        };
        self.lay(carcass.clone());
        Some(carcass)
    }
}
