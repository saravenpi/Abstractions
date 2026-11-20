//! The watch of the signs: rolled again by a task while the old one answers, and read once a second.

use std::sync::Arc;

use bevy::math::DVec3;

use super::Trail;
use super::tune::{MOVE, RADIUS, ROLL};
use crate::fauna::herds::{Land, herd_arc, read};
use crate::fauna::roll::{Rolling, roll};
use crate::life::herd::Search;
use crate::life::signs;
use crate::universe::PlanetSpec;
use crate::world::Globe;

impl Trail {
    /// Returns true when the watch must be rolled again for the robot at unit direction `at` on the
    /// globe of seed `seed` and radius `radius` metres at universe `time`.
    fn stale(&self, (seed, radius): (u64, f64), at: DVec3, time: f64) -> bool {
        let (was, from, then) = self.rolled;
        let far = herd_arc(from, at) * radius > MOVE;
        self.watch.is_none() || was != seed || far || !(then..then + ROLL).contains(&time)
    }

    /// Swaps in the watch a task finished rolling, or rolls again when needed: at once when there is
    /// no watch or it is of another globe, else in a task while the old watch answers. A watch that
    /// no longer covers `time` is waited for.
    fn refresh(&mut self, (globe, planet): (&Arc<Globe>, &Arc<PlanetSpec>), at: DVec3, time: f64) {
        let (seed, radius) = (globe.seed(), globe.radius());
        let lapsed = !self.watch.as_ref().is_some_and(|w| w.covers(time));
        if let Some(rolling) = self.rolling.take().filter(|r| r.seed == seed) {
            match rolling.finish(lapsed) {
                Ok(Some((watch, from, then))) => {
                    (self.watch, self.rolled) = (Some(watch), (seed, from, then));
                }
                Ok(None) => {}
                Err(rolling) => self.rolling = Some(rolling),
            }
        }
        if self.rolling.is_some() || !self.stale((seed, radius), at, time) {
            return;
        }
        let reach = RADIUS + signs::tune::REACH + MOVE;
        let search = Search {
            at,
            radius: reach,
            planet: radius,
            time,
        };
        if self.watch.is_none() || self.rolled.0 != seed {
            self.watch = roll(globe, planet, &search);
            self.rolled = (seed, at, time);
        } else {
            self.rolling = Some(Rolling::start(globe, planet, search));
        }
    }

    /// Rolls the watch again when needed, then reads the signs around unit direction `at` on the
    /// globe and planet at universe `time`.
    pub fn read(&mut self, owned: (&Arc<Globe>, &Arc<PlanetSpec>), at: DVec3, time: f64) {
        let (globe, planet) = owned;
        let Some(life) = globe.life() else {
            return;
        };
        let land = Land {
            globe,
            life,
            planet,
        };
        self.refresh(owned, at, time);
        let Some(watch) = self.watch.as_mut() else {
            return;
        };
        let search = Search {
            at,
            radius: RADIUS,
            planet: globe.radius(),
            time,
        };
        let out = &mut self.signs;
        read(land, time, |g| {
            signs::near(land.life, g, watch, &search, out)
        });
    }
}
