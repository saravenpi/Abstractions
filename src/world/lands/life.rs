//! What may live on the lands of each class: how thick the plants grow, and what lives on the sea floor.

use super::Lands;
use crate::universe::PlanetClass;
use crate::world::propmesh::Plant;
use crate::world::scatter::plants_for;
use crate::world::{Biome, Surface};

impl Lands {
    /// Returns the share of the plant tries of home that the planet gets, 0 where nothing grows.
    pub fn growth(self) -> f32 {
        match self.class {
            PlanetClass::Ocean | PlanetClass::Island => 1.0,
            PlanetClass::Desert => 0.5,
            PlanetClass::Ice => 0.4,
            PlanetClass::Lava | PlanetClass::Barren | PlanetClass::GasGiant => 0.0,
        }
    }

    /// Returns true when the sea floor carries life: kelp, seagrass, coral and shells.
    pub fn sea_life(self) -> bool {
        self.sea && self.temperate()
    }

    /// Returns the plants that may stand on `surface` in `biome`, with their chance per try: the table of
    /// home, and nothing on a sea floor without life.
    pub fn plants(self, biome: Biome, surface: Surface) -> &'static [(Plant, f32)] {
        let floor = matches!(surface, Surface::Water | Surface::Shallows | Surface::Reef);
        match floor && !self.sea_life() {
            true => &[],
            false => plants_for(biome, surface),
        }
    }
}
