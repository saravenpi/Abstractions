//! The bodies poses are converted against, looked up once per star and kept, and the two
//! questions asked of them: which wire frame a pose goes in, and where it lands in another frame.

use std::collections::HashMap;

use abstractions_net::wire;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::frame::{Place, Pose, Rigid};
use crate::universe::{BodyId, PlanetSpec, StarId, Universe};

/// Radii from the centre of a body within which a flying ship sent around it stays around it
/// rather than going in the frame of its star: the reach within which the flight carries a ship
/// along with a planet.
pub const REACH: f64 = 100.0;

/// Radii from the centre of a body within which a flying ship not yet around it goes around
/// it. Under [`REACH`], so a ship on the edge does not flip between two frames every record.
pub const ENTER: f64 = 95.0;

/// The planets and moons of the stars seen so far, and where those stars are in their galaxy.
#[derive(Resource, Debug, Default)]
pub struct Bodies {
    /// The universe the bodies belong to.
    universe: Option<Universe>,
    /// The planets of each star, then their moons.
    systems: HashMap<StarId, Vec<PlanetSpec>>,
    /// Where each star is in its galaxy, when it exists.
    stars: HashMap<StarId, Option<DVec3>>,
    /// The body the last flying thing was sent around, which it leaves only past [`REACH`].
    held: Option<BodyId>,
}

impl Bodies {
    /// Forgets every body when the universe is not the one they were looked up in.
    pub fn sync(&mut self, universe: &Universe) {
        if self.universe != Some(*universe) {
            *self = Self {
                universe: Some(*universe),
                ..Self::default()
            };
        }
    }

    /// Returns the planets and moons of `star`, looked up the first time. None before
    /// [`Bodies::sync`].
    pub fn system(&mut self, star: StarId) -> &[PlanetSpec] {
        let Some(universe) = self.universe else {
            return &[];
        };
        self.systems.entry(star).or_insert_with(|| {
            let planets = universe.system(star).planets;
            let moons = planets.iter().filter(|planet| planet.moons > 0);
            let moons: Vec<PlanetSpec> = moons.flat_map(|p| universe.moons(p.id)).collect();
            planets.into_iter().chain(moons).collect()
        })
    }

    /// Returns how the frame `place` sits in the frame of `star` at `time`, or nothing when it
    /// lies around another star.
    fn rigid(&mut self, place: Place, star: StarId, time: f64) -> Option<Rigid> {
        let (body, turning) = match place {
            Place::System(at) => return (at == star).then_some(Rigid::STILL),
            Place::Galaxy(galaxy) => return self.galaxy(star).filter(|_| galaxy == star.galaxy),
            Place::Surface(body) => (body, true),
            Place::Around(body) => (body, false),
        };
        let spec = self.system(star).iter().find(|spec| spec.id == body)?;
        Some(Rigid::body(spec, time, turning))
    }

    /// Returns how the frame of the galaxy of `star` sits in the frame of the star.
    fn galaxy(&mut self, star: StarId) -> Option<Rigid> {
        let universe = self.universe?;
        let at = *self
            .stars
            .entry(star)
            .or_insert_with(|| universe.star(star).map(|spec| spec.pos));
        at.map(|at| Rigid {
            offset: -at,
            ..Rigid::STILL
        })
    }

    /// Returns `pose` of the frame `from` in the frame `to` at universe time `time`, or nothing
    /// when the two frames lie around different stars.
    pub fn convert(&mut self, pose: Pose, from: Place, to: Place, time: f64) -> Option<Pose> {
        if from == to {
            return Some(pose);
        }
        let star = star_of(from).or(star_of(to))?;
        let out = self.rigid(from, star, time)?;
        let back = self.rigid(to, star, time)?;
        Some(back.back(&out.out(&pose)))
    }

    /// Returns the wire frame for a thing at `pos` in the frame `place` at `time`: the surface
    /// when it is `fixed` to it, around the body it flies at, the star frame in deep space,
    /// and between the stars the frame of the `last` star it was at.
    pub fn choose(
        &mut self,
        (place, pos): (Place, DVec3),
        fixed: bool,
        time: f64,
        last: Option<StarId>,
    ) -> Option<wire::Frame> {
        match place {
            Place::Surface(body) if fixed => Some(wire::Frame::Surface(body)),
            Place::Surface(body) | Place::Around(body) => Some(wire::Frame::Around(body)),
            Place::System(star) => Some(self.near(star, pos, time)),
            Place::Galaxy(galaxy) => last
                .filter(|star| star.galaxy == galaxy)
                .map(wire::Frame::System),
        }
    }

    /// Returns the frame around the body of `star` whose surface is nearest `pos`, among those
    /// within [`ENTER`] radii and the one held within [`REACH`], else the frame of the star.
    fn near(&mut self, star: StarId, pos: DVec3, time: f64) -> wire::Frame {
        let held = self.held.filter(|body| body.star == star);
        let gap = |spec: &PlanetSpec| spec.position(time).distance(pos) - spec.radius;
        let reach = |spec: &PlanetSpec| if Some(spec.id) == held { REACH } else { ENTER };
        let close = |spec: &&PlanetSpec| gap(spec) < (reach(spec) - 1.0) * spec.radius;
        let bodies = self.system(star).iter().filter(close);
        let nearest = bodies
            .min_by(|a, b| gap(a).total_cmp(&gap(b)))
            .map(|spec| spec.id);
        self.held = nearest;
        nearest.map_or(wire::Frame::System(star), wire::Frame::Around)
    }
}

/// Returns the star a frame lies around, nothing for the frame of a galaxy.
pub fn star_of(place: Place) -> Option<StarId> {
    match place {
        Place::Surface(body) | Place::Around(body) => Some(body.star),
        Place::System(star) => Some(star),
        Place::Galaxy(_) => None,
    }
}
