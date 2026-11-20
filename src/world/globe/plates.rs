//! Plates: the crust of a planet split into 8 to 20 plates, each continental or oceanic and each turning
//! about its own axis, with the ranges, trenches, arcs, ridges and rifts their edges make.
//!
//! The split is a Voronoi diagram of seeds on a jittered Fibonacci lattice, read at a warped direction so
//! that the edges bend. Everything is a pure function of the direction and holds no cache: one sample
//! reads three noise values and one dot product per plate. The method follows Cortial et al.,
//! "Procedural Tectonic Planets", and Frozen Fractal's simplified plates, both listed in
//! `handoff/PLANETS.md`.

mod edge;
mod interior;
mod marks;
mod meet;
mod near;
mod seeds;
mod tune;
mod warp;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_variety;

pub use edge::Boundary;
pub use interior::{Interior, age};
pub use tune::RANGE;

use bevy::math::DVec3;

use crate::world::islands::sub_seed;
use marks::Marks;
use near::Near;
use tune::{COUNT, MARGIN};

/// The crust a plate carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crust {
    /// Thick, light crust that stands high: the continents.
    Continental,
    /// Thin, dense crust that lies low: the ocean floors, or the dry basins of a world without a sea.
    Oceanic,
}

/// What the plates say about one direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Index of the plate that holds the direction.
    pub plate: u8,
    /// Crust of that plate.
    pub crust: Crust,
    /// The crust blended widely across the plate edges: 1 well inside a continent, -1 well inside a sea
    /// plate, 0 on an edge between the two.
    pub land: f32,
    /// Radians to the nearest edge of the plate.
    pub edge: f32,
    /// What the plates do along that edge.
    pub boundary: Boundary,
    /// Height the edges add on land, as a share of the highest mountains. Negative in a rift.
    pub rise: f32,
    /// Height the edges add to the sea floor, as a share of the ocean depth. Negative in a trench.
    pub floor: f32,
    /// Height of the volcanoes, as a share of the highest mountains.
    pub cone: f32,
    /// Height of the impact basins in metres, negative in a bowl.
    pub pit: f32,
}

/// The plates of one planet.
#[derive(Debug, Clone, PartialEq)]
pub struct Plates {
    /// Seed of the warp noise.
    seed: u32,
    /// Unit direction of the centre of each plate.
    centres: Vec<DVec3>,
    /// Crust of each plate.
    crust: Vec<Crust>,
    /// Turn of each plate, see [`seeds::spins`].
    spins: Vec<DVec3>,
    /// Distance between the centres of every two plates, row by row.
    chords: Vec<f64>,
    /// Distance between the centres of two neighbour plates on an even lattice.
    spacing: f64,
    /// What moves the plates.
    interior: Interior,
    /// Volcanoes and impact basins.
    marks: Marks,
}

impl Plates {
    /// Builds the plates of planet `seed` of `radius` metres with `interior`. The same values always give
    /// the same plates.
    pub fn new(seed: u64, radius: f64, interior: Interior) -> Self {
        let count = seeds::count(seed, &interior);
        let centres = seeds::centres(seed, count);
        let chord = |a: &DVec3| {
            centres
                .iter()
                .map(|b| (*a - *b).length())
                .collect::<Vec<_>>()
        };
        Self {
            seed: sub_seed(seed, 30),
            chords: centres.iter().flat_map(chord).collect(),
            crust: seeds::crusts(seed, count, interior.lowland),
            spins: seeds::spins(seed, count),
            spacing: seeds::spacing(count),
            marks: Marks::new(seed, radius, &interior),
            centres,
            interior,
        }
    }

    /// Returns how many plates the planet has.
    pub fn count(&self) -> usize {
        self.centres.len()
    }

    /// Returns what moves the plates.
    pub fn interior(&self) -> &Interior {
        &self.interior
    }

    /// Returns what the plates say about unit direction `dir`.
    pub fn sample(&self, dir: DVec3) -> Sample {
        let w = warp::warped(self.seed, dir);
        let near = self.near(w);
        let lift = self.uplift(w, &near);
        let (cone, pit) = self.marks.at(dir);
        Sample {
            plate: near.ids[0] as u8,
            crust: self.crust[near.ids[0]],
            land: self.land(&near),
            edge: near.edge as f32,
            boundary: Boundary::of(self.closing(w, near.ids[0], near.next)),
            rise: lift.rise,
            floor: lift.floor,
            cone,
            pit,
        }
    }

    /// Returns the motion of `plate` at unit direction `dir`: a vector along the ground, as long as the
    /// speed of the plate there over the speed of the fastest plate of the fastest planet. A planet whose
    /// interior is cold gives zero.
    pub fn motion(&self, plate: u8, dir: DVec3) -> DVec3 {
        self.spins[usize::from(plate)].cross(dir) * f64::from(self.interior.mobility)
    }

    /// Returns the plates near warped unit direction `w`: the one whose centre is closest, then those
    /// whose edge with it is within reach.
    fn near(&self, w: DVec3) -> Near {
        let mut dots = [f64::MIN; COUNT.1];
        let mut top = 0;
        for (k, centre) in self.centres.iter().enumerate() {
            dots[k] = centre.dot(w);
            if dots[k] > dots[top] {
                top = k;
            }
        }
        let mut near = Near::of(top, dots[top]);
        for k in (0..self.count()).filter(|&k| k != top) {
            near.take(k, dots[k], [self.chord(top, k), self.spacing]);
        }
        near
    }

    /// Returns the crust blended over the plates of `near`: each plate weighs by how close its edge is, the
    /// owner in full, so the value is the mean of both crusts on an edge and reaches the owner's own
    /// [`MARGIN`] inside its cell.
    fn land(&self, near: &Near) -> f32 {
        let (mut sum, mut total) = (0.0, 0.0);
        for k in 0..near.count {
            let gap = (near.dots[0] - near.dots[k]) / (self.spacing * MARGIN);
            let weight = (1.0 - gap).max(0.0).powi(2);
            let level = [-1.0, 1.0][usize::from(self.crust[near.ids[k]] == Crust::Continental)];
            sum += weight * level;
            total += weight;
        }
        (sum / total) as f32
    }

    /// Returns the distance between the centres of plates `i` and `j`.
    fn chord(&self, i: usize, j: usize) -> f64 {
        self.chords[i * self.count() + j]
    }
}
