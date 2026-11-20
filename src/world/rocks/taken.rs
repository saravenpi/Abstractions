//! The mass taken from each rock, and how it is kept in the world file.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;

use super::RockId;
use super::tune::LEAST;

/// What was taken from one rock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quarry {
    /// Mass taken so far, in kilograms.
    pub taken: f32,
    /// Mass of the whole rock, in kilograms.
    pub whole: f32,
}

/// The mass the players have taken from the rocks of every planet of the world.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Taken(Arc<HashMap<RockId, Quarry>>);

impl Taken {
    /// Returns the mass of the whole rock `id` when it was quarried, in kilograms.
    pub fn whole(&self, id: RockId) -> Option<f32> {
        self.0.get(&id).map(|q| q.whole)
    }

    /// Returns the mass taken from rock `id`, in kilograms.
    pub fn kilograms(&self, id: RockId) -> f32 {
        self.0.get(&id).map_or(0.0, |q| q.taken)
    }

    /// Returns the share of the mass of rock `id` that is left, from 0 to 1: 1 for a rock nobody
    /// touched, 0 for a rock that is gone.
    pub fn left(&self, id: RockId) -> f32 {
        let share = |q: &Quarry| ((q.whole - q.taken) / q.whole).clamp(0.0, 1.0);
        self.0.get(&id).map_or(1.0, share)
    }

    /// Notes that `kilograms` came off rock `id`, whose whole mass is `whole` kilograms. What is left
    /// under [`LEAST`] counts as taken.
    pub fn quarry(&mut self, id: RockId, kilograms: f32, whole: f32) {
        let entry = Arc::make_mut(&mut self.0)
            .entry(id)
            .or_insert(Quarry { taken: 0.0, whole });
        entry.taken = (entry.taken + kilograms).min(entry.whole);
        if entry.whole - entry.taken <= LEAST {
            entry.taken = entry.whole;
        }
    }

    /// Returns the quarries as the world file keeps them, `id:taken:whole` with the id in 16
    /// hexadecimal digits, in order of id.
    pub fn to_text(&self) -> Vec<String> {
        let mut rows: Vec<(&RockId, &Quarry)> = self.0.iter().collect();
        rows.sort_unstable_by_key(|(id, _)| **id);
        let row = |(id, q): (&RockId, &Quarry)| format!("{}:{}:{}", id.text(), q.taken, q.whole);
        rows.into_iter().map(row).collect()
    }

    /// Returns the quarries the world file's `text` keeps. A row that is just an id, as the first
    /// files wrote, is a rock taken whole. Any other row that does not read is skipped.
    pub fn from_text(text: &[String]) -> Self {
        let rows = text.iter().filter_map(|t| read(t));
        Self(Arc::new(rows.collect()))
    }
}

/// Reads one row of the world file.
fn read(row: &str) -> Option<(RockId, Quarry)> {
    let mut parts = row.split(':');
    let id = RockId::parse(parts.next()?)?;
    let Some(taken) = parts.next() else {
        return Some((
            id,
            Quarry {
                taken: 1.0,
                whole: 1.0,
            },
        ));
    };
    let (taken, whole): (f32, f32) = (taken.parse().ok()?, parts.next()?.parse().ok()?);
    let sound = whole.is_finite() && whole > 0.0 && (0.0..=whole).contains(&taken);
    sound.then_some((id, Quarry { taken, whole }))
}
