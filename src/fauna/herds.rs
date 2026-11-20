//! The herds around the local robot: one [`Watch`], rolled again before it stops answering or when the
//! robot walks far from where it was rolled, and what it shows at the last scan.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::site;
use super::tune::{WATCH_MOVE, WATCH_RADIUS, WATCH_SPARE};
use crate::life::herd::{Ground, Herd, Ledger, Search, Watch};
use crate::life::hunt::{Carcass, Hunt};
use crate::life::{Life, herd};
use crate::universe::PlanetSpec;
use crate::world::Globe;

/// The planet the herds are read on: its globe and its record of orbit and spin.
#[derive(Clone, Copy)]
pub struct Land<'a> {
    /// The globe of the planet.
    pub globe: &'a Globe,
    /// The life of the planet.
    pub life: &'a Life,
    /// The planet's orbit and spin, for its daylight.
    pub planet: &'a PlanetSpec,
}

/// The watch over the herds around the local robot and what it showed at the last scan.
#[derive(Resource, Default)]
pub struct Herds {
    /// The watch, once rolled.
    watch: Option<Watch>,
    /// Seed of the globe the watch was rolled on.
    seed: u64,
    /// Unit direction the watch was rolled around.
    at: DVec3,
    /// Herds within the search radius at the last scan.
    pub herds: Vec<Herd>,
    /// Hunts under way or with meat left at the last scan.
    pub hunts: Vec<Hunt>,
    /// Carcasses within the search radius at the last scan.
    pub carcasses: Vec<Carcass>,
    /// Members killed by players, gone from their herds until they recover.
    pub ledger: Ledger,
}

/// Runs `f` with the herds' view of `land` at universe `time`.
pub fn read<T>(land: Land, time: f64, f: impl FnOnce(&Ground) -> T) -> T {
    let sample = |dir: DVec3| site::site(land.globe, land.planet, dir, time);
    let light = |dir: DVec3, at: f64| site::light(land.planet, dir, at);
    let height = |dir: DVec3| land.globe.height(dir);
    f(&Ground {
        sample: &sample,
        height: &height,
        light: &light,
    })
}

impl Herds {
    /// Forgets the watch and everything it showed.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Returns true when the watch must be rolled again for the robot at unit direction `at` on
    /// `land` at universe `time`.
    fn stale(&self, land: Land, at: DVec3, time: f64) -> bool {
        let far = herd_arc(self.at, at) * land.globe.radius() > WATCH_MOVE;
        let late = !self
            .watch
            .as_ref()
            .is_some_and(|w| w.covers(time) && w.covers(time + WATCH_SPARE));
        late || self.seed != land.globe.seed() || far
    }

    /// Rolls the watch again when needed, then reads the herds, hunts and carcasses around unit
    /// direction `at` on `land` at universe `time`.
    pub fn scan(&mut self, land: Land, at: DVec3, time: f64) {
        if self.stale(land, at, time) {
            let search = Search {
                at,
                radius: WATCH_RADIUS,
                planet: land.globe.radius(),
                time,
            };
            self.watch = Some(read(land, time, |g| Watch::new(land.life, g, &search)));
            (self.seed, self.at) = (land.globe.seed(), at);
        }
        let Some(watch) = self.watch.as_mut() else {
            return;
        };
        self.herds = read(land, time, |g| watch.herds(land.life, g, time));
        self.hunts = watch.hunts(land.life, time);
        self.carcasses = watch.carcasses(land.life, time);
        self.ledger.prune(time);
    }

    /// Returns herd `id` at universe `time`, wherever it has gone, or nothing when the watch does not
    /// hold it or does not answer for that time.
    pub fn herd(&mut self, land: Land, id: u64, time: f64) -> Option<Herd> {
        let watch = self.watch.as_mut().filter(|w| w.covers(time))?;
        read(land, time, |g| watch.herd(land.life, g, id, time))
    }
}

/// Returns the angle in radians between unit directions `a` and `b`.
pub fn herd_arc(a: DVec3, b: DVec3) -> f64 {
    a.cross(b).length().atan2(a.dot(b))
}

/// Returns member `slot` of `herd` at `time` and its unit direction on a planet of `radius` metres,
/// or nothing when that slot is empty or dead.
pub fn member(herd: &Herd, slot: u8, time: f64, radius: f64) -> Option<(herd::Member, DVec3)> {
    let member = herd::member(herd, slot, time)?;
    Some((
        member,
        super::place::beside(herd.centre, member.offset, radius),
    ))
}
