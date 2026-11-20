//! What the universe says about a galaxy, a star, a planet and a whole system.

use bevy::math::{DVec3, Vec3};

use super::ids::{BodyId, GalaxyId, StarId};
use super::record::PlanetRecord;

/// Newton's constant in SI units, linking a star mass in kilograms to the pull it exerts.
pub const GRAVITATION: f64 = 6.674e-11;

/// The overall shape of a galaxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GalaxyKind {
    /// A flat disc with swirled arms.
    Spiral,
    /// A featureless ball of stars.
    Elliptical,
    /// A loose cloud without structure.
    Irregular,
}

/// One galaxy.
#[derive(Debug, Clone, PartialEq)]
pub struct GalaxySpec {
    /// Identifier, the grid cell of the galaxy.
    pub id: GalaxyId,
    /// Seed every star of the galaxy derives from.
    pub seed: u64,
    /// Display name.
    pub name: String,
    /// Overall shape.
    pub kind: GalaxyKind,
    /// Number of arms, 0 unless the galaxy is a spiral.
    pub arms: u8,
    /// Radius of the star disc in metres.
    pub radius: f64,
    /// Centre of the galaxy in metres from the minimum corner of its grid cell.
    ///
    /// Star positions and sectors are measured from this point.
    pub center: DVec3,
}

/// Spectral class of a star, from hottest to coolest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StarClass {
    /// Blue, very hot and rare.
    O,
    /// Blue white.
    B,
    /// White.
    A,
    /// Yellow white.
    F,
    /// Yellow, like the Sun.
    G,
    /// Orange.
    K,
    /// Red, cool and common.
    M,
}

/// One star.
#[derive(Debug, Clone, PartialEq)]
pub struct StarSpec {
    /// Identifier.
    pub id: StarId,
    /// Seed the planets of the star derive from.
    pub seed: u64,
    /// Display name.
    pub name: String,
    /// Spectral class.
    pub class: StarClass,
    /// Mass in kilograms. Multiply by [`GRAVITATION`] for the pull, see [`StarSpec::mu`].
    pub mass: f64,
    /// Light output in luminosities of the Sun.
    pub luminosity: f64,
    /// Radius in metres.
    pub radius: f64,
    /// Surface temperature in kelvin.
    pub temperature: f64,
    /// Age in seconds, shared by every planet and moon of the star.
    pub age: f64,
    /// Linear RGB colour of the light, each channel from 0 to 1.
    pub color: Vec3,
    /// Position in metres from the galaxy centre.
    pub pos: DVec3,
}

impl StarSpec {
    /// Returns the gravitational parameter in cubic metres per second squared.
    pub fn mu(&self) -> f64 {
        GRAVITATION * self.mass
    }
}

/// A circular orbit around a star or a planet. Seen from +Y it turns counterclockwise, from +X
/// toward -Z, the way a body with a positive day spins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    /// Radius in metres.
    pub radius: f64,
    /// Seconds for one turn.
    pub seconds: f64,
    /// Angle along the orbit at time 0, in radians.
    pub phase: f64,
}

impl Orbit {
    /// Returns the position at `time` seconds in metres from the centre, on the XZ plane.
    pub fn at(&self, time: f64) -> DVec3 {
        let angle = self.phase + std::f64::consts::TAU * time / self.seconds;
        DVec3::new(angle.cos(), 0.0, -angle.sin()) * self.radius
    }
}

/// The rings of a planet, flat in the plane of its equator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rings {
    /// Inner radius in metres from the centre of the planet, 1.2 to 1.6 planet radii and above
    /// the air.
    pub inner: f64,
    /// Outer radius in metres from the centre of the planet, at least 0.3 planet radii past the
    /// inner one and at most 2.4: inside the Roche limit and inside the innermost moon.
    pub outer: f64,
    /// Share of the light the rings stop, from 0.2 to 0.9.
    pub density: f32,
    /// Linear RGB colour of the ring material: pale ice on a planet under 220 K, dust above.
    pub color: bevy::math::Vec3,
}

/// One planet or moon. For a moon the orbit fields describe its turn around its planet, and
/// [`PlanetSpec::host`] the orbit of that planet around the star.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetSpec {
    /// Identifier.
    pub id: BodyId,
    /// Seed of the terrain.
    pub seed: u64,
    /// Display name.
    pub name: String,
    /// Sea level radius in metres.
    pub radius: f64,
    /// Surface gravity in metres per second squared.
    pub gravity: f64,
    /// Seconds for one turn around its own +Y axis against the stars: the physical spin, the same
    /// way round as its orbit. Equal to `orbit_seconds` on a tidally locked body. The day from
    /// noon to noon follows from this and the orbit: the day clock of the renderer works it out.
    pub day_seconds: f64,
    /// Radius of the circular orbit around the star in metres.
    pub orbit_radius: f64,
    /// Seconds for one orbit.
    pub orbit_seconds: f64,
    /// Angle along the orbit at time 0, in radians.
    pub orbit_phase: f64,
    /// Height above sea level in metres where the air ends.
    pub atmosphere_height: f64,
    /// Height above sea level in metres of the bottom of the cloud layer.
    pub cloud_base: f64,
    /// Height above sea level in metres of the top of the cloud layer.
    pub cloud_top: f64,
    /// What the physics says about its air, heat and water, and the class that follows.
    pub record: PlanetRecord,
    /// For a moon, the orbit of its planet around the star. Nothing for a planet.
    pub host: Option<Orbit>,
    /// Number of moons: up to 6 for a gas giant, 0 to 2 for a solid planet, 0 for a moon.
    pub moons: u8,
    /// The rings, if the planet has any.
    pub rings: Option<Rings>,
}

/// Shares of the Hill radius of a body between which it stops carrying the warp bubble: it carries
/// it whole inside the first and not at all outside the second. The drive and the sky read it.
pub const CARRY: (f64, f64) = (0.25, 0.5);

impl PlanetSpec {
    /// Returns the Hill radius in metres around what it goes round: `a (m / 3 M)^(1/3)`.
    ///
    /// The gravitational parameter of what it goes round follows from its orbit, which is
    /// circular: `mu = (2 pi / T)^2 a^3`.
    pub fn hill_radius(&self) -> f64 {
        let (a, rate) = (
            self.orbit_radius,
            std::f64::consts::TAU / self.orbit_seconds,
        );
        a * (self.mu() / (3.0 * rate * rate * a * a * a)).cbrt()
    }

    /// Returns the gravitational parameter in cubic metres per second squared.
    pub fn mu(&self) -> f64 {
        self.gravity * self.radius * self.radius
    }

    /// Returns true when tides have locked the spin to the orbit: the body always shows the same
    /// face to what it goes round, its star for a planet and its planet for a moon.
    pub fn locked(&self) -> bool {
        self.day_seconds == self.orbit_seconds
    }

    /// Returns its own orbit: around the star for a planet, around its planet for a moon.
    pub fn orbit(&self) -> Orbit {
        Orbit {
            radius: self.orbit_radius,
            seconds: self.orbit_seconds,
            phase: self.orbit_phase,
        }
    }

    /// Returns the position at `time` seconds in metres from the star, on the XZ plane of the
    /// system. A moon is where its planet is, plus its own place on its orbit around it.
    pub fn position(&self, time: f64) -> DVec3 {
        let around = self.host.map_or(DVec3::ZERO, |host| host.at(time));
        around + self.orbit().at(time)
    }
}

/// A star with its planets, ordered outward.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemSpec {
    /// The star at the origin of the system frame.
    pub star: StarSpec,
    /// The planets, `planets[i].id.index == i`.
    pub planets: Vec<PlanetSpec>,
}
