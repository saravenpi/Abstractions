//! One row of the survey: everything measured on the home planet of one universe seed.

use std::sync::Arc;

use bevy::math::DVec3;

use super::herd::{herd, walker};
use super::shares::{Shares, climate_at, shares};
use crate::climate::{self, Climate, Spot};
use crate::home::Base;
use crate::life::flora::KELP;
use crate::life::{self, Guild, Life, Species};
use crate::universe::{AU, PlanetClass, PlanetSpec, StarSpec, Universe, Water, Zone};
use crate::world::globe::MEAN_SEASON;
use crate::world::{Biome, Globe};

/// What was measured on one home.
pub struct Row {
    /// Universe seed.
    pub seed: u64,
    /// The star.
    pub star: StarSpec,
    /// The planet.
    pub planet: PlanetSpec,
    /// The life of the planet, if it lives.
    pub life: Option<Life>,
    /// Mean temperature at the equator and at the pole in kelvin.
    pub span: (f32, f32),
    /// Annual mean climate at the spawn.
    pub spawn: Climate,
    /// Biome at the spawn.
    pub biome: Biome,
    /// Shares of the land.
    pub shares: Shares,
    /// Whether a herd of a walking species lives within [`WALK`] of the spawn.
    pub herd: bool,
}

/// Returns true for a land plant species.
fn plant(s: &Species) -> bool {
    s.role.guild == Guild::Producer && s.archetype != KELP
}

/// Returns the row of the home planet of universe `seed`.
pub fn measure(seed: u64) -> Row {
    let universe = Universe::new(seed);
    let body = universe.home();
    let planet = universe.planet(body);
    let star = universe.system(body.star).star;
    let life = life::of(&planet, &star);
    let globe = Arc::new(Globe::of(&planet).with_life(life.clone()));
    let dir = Base::landing(&globe).normalize();
    let spot = |dir: DVec3| Spot {
        dir,
        altitude: 0.0,
        inland: 0.5,
        slope: bevy::math::Vec2::ZERO,
    };
    let at = |dir: DVec3| climate::at(globe.setting(), &spot(dir), MEAN_SEASON).temperature;
    let walks = life.as_ref().is_some_and(|l| herd(&globe, &planet, l, dir));
    Row {
        seed,
        span: (at(DVec3::X), at(DVec3::Y)),
        spawn: climate_at(&globe, dir),
        biome: globe.biome(dir),
        shares: shares(&globe),
        herd: walks,
        star,
        planet,
        life,
    }
}

impl Row {
    /// Returns how many plant species and animal species the life gives.
    pub fn species(&self) -> (usize, usize) {
        let all = self.life.iter().flat_map(|l| l.species.iter());
        let plants = all.clone().filter(|s| plant(s)).count();
        (plants, all.filter(|s| walker(s)).count())
    }

    /// Returns true when the world has liquid water on its surface.
    pub fn wet(&self) -> bool {
        let record = &self.planet.record;
        record.water == Water::Liquid && record.ocean > 0.05
    }

    /// Returns true when plants stand at the spawn: land of a class that grows them, a biome that is
    /// not desert, no snow, and a plant species in the life.
    pub fn plants(&self) -> bool {
        let class = self.planet.record.class;
        let grows = !matches!(
            class,
            PlanetClass::Lava | PlanetClass::Barren | PlanetClass::GasGiant | PlanetClass::Desert
        );
        let cold = climate::snow(&self.spawn);
        grows && !cold && self.biome != Biome::Desert && self.species().0 > 0
    }

    /// Returns true when the spawn is between 0 and 40 C and the world is wet, green and walked.
    pub fn good(&self) -> bool {
        let warm = (273.15..=313.15).contains(&self.spawn.temperature);
        self.wet() && self.plants() && self.species().1 > 0 && warm
    }

    /// Returns where the orbit lies against the habitable zone, in a word.
    pub fn zone(&self) -> &'static str {
        match self.planet.record.zone {
            Zone::Hot => "hot",
            Zone::Habitable => "hab",
            Zone::Cold => "cold",
        }
    }

    /// Returns the orbit in astronomical units.
    pub fn au(&self) -> f64 {
        self.planet.orbit_radius / AU
    }
}
