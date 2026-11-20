//! A scene: every herd that can matter around one search, rolled once into a [`Store`], with the
//! waypoints it reads.
//!
//! A herd that can be seen within the search radius has its home within the radius plus its roam.
//! Hunters are rolled out to [`TERRITORY`] past those homes, so every hunter that can hunt such a
//! herd is known, and plant eaters out to [`TERRITORY`] past those hunters, so every herd such a
//! hunter can choose from is known. Waypoints are kept in the store, so a herd asked for twice, or at
//! two times of one leg, costs its site samples once.

use std::cell::RefCell;
use std::collections::HashMap;

use bevy::math::DVec3;

use super::cells::Grid;
use super::roll::{self, MAX_ROAM, Roam};
use super::route::{self, Pose};
use super::sphere::arc;
use super::{Ground, Search};
use crate::life::census::Site;
use crate::life::hunt::TERRITORY;
use crate::life::{Life, Species};

/// Side in metres of the cells hunters are rolled in.
const HUNTER_SIDE: f64 = 1024.0;

/// Waypoints drawn so far and their sites, by herd and leg.
type Ways = HashMap<(u64, i64), (DVec3, Site)>;

/// The herds rolled around one search and the waypoints drawn for them so far.
#[derive(Debug, Clone)]
pub(in crate::life) struct Store {
    /// Sea level radius of the planet in metres.
    pub planet: f64,
    /// Every herd, plant eaters first, in a stable order.
    pub roams: Vec<Roam>,
    /// Waypoints drawn so far, by herd and leg.
    pub ways: Ways,
}

impl Store {
    /// Rolls the herds of `life` that can matter within the radius of `search`.
    pub fn roll(life: &Life, ground: &Ground, search: &Search) -> Self {
        Self {
            planet: search.planet,
            roams: roams(life, ground, search),
            ways: HashMap::new(),
        }
    }
}

/// Every herd around one search, read through its store.
pub(in crate::life) struct Scene<'a> {
    /// The life of the planet.
    pub life: &'a Life,
    /// The planet as herds read it.
    pub ground: &'a Ground<'a>,
    /// Sea level radius of the planet in metres.
    pub planet: f64,
    /// Mean side of a plant eater cell in metres.
    pub side: f64,
    /// Every herd, plant eaters first, in a stable order.
    pub roams: &'a [Roam],
    /// Waypoints drawn so far, by herd and leg.
    ways: RefCell<&'a mut Ways>,
}

impl<'a> Scene<'a> {
    /// Opens the herds of `store` on `life` and `ground`; new waypoints go into the store.
    pub fn open(life: &'a Life, ground: &'a Ground<'a>, store: &'a mut Store) -> Self {
        Self {
            life,
            ground,
            planet: store.planet,
            side: Grid::of(store.planet).side(),
            roams: &store.roams,
            ways: RefCell::new(&mut store.ways),
        }
    }

    /// Returns the species of `roam`.
    pub fn species(&self, roam: &Roam) -> Option<&'a Species> {
        self.life.species.get(usize::from(roam.species))
    }

    /// Returns the distance in metres along the ground between unit directions `a` and `b`.
    pub fn metres(&self, a: DVec3, b: DVec3) -> f64 {
        arc(a, b) * self.planet
    }

    /// Returns waypoint `leg` of `roam` of `species`, and its site, drawn once per scene.
    pub fn waypoint(&self, roam: &Roam, species: &Species, leg: i64) -> (DVec3, Site) {
        if let Some(way) = self.ways.borrow().get(&(roam.id, leg)) {
            return *way;
        }
        let way = route::draw(roam, species, leg, self.planet, self.ground.sample);
        self.ways.borrow_mut().insert((roam.id, leg), way);
        way
    }

    /// Returns where `roam` is on its route at `time`, before any hunt bends it.
    pub fn pose(&self, roam: &Roam, time: f64) -> Option<Pose> {
        Some(route::pose(self, roam, self.species(roam)?, time))
    }
}

/// Returns every herd of `life` that can matter within the radius of `search`: plant eaters, then hunters.
fn roams(life: &Life, ground: &Ground, search: &Search) -> Vec<Roam> {
    let (fine, coarse) = (
        Grid::of(search.planet),
        Grid::sized(search.planet, HUNTER_SIDE),
    );
    let seen = search.radius + MAX_ROAM + fine.side();
    let grazing = seen + 2.0 * TERRITORY;
    let hunting = seen + TERRITORY + coarse.side();
    let sample = |dir| (ground.sample)(dir);
    let mut roams = roll::around(life, &fine, (search.at, grazing), &sample, false);
    roams.extend(roll::around(
        life,
        &coarse,
        (search.at, hunting),
        &sample,
        true,
    ));
    roams
}
