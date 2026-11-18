//! The vantage: what the sky systems know about where the camera is, refreshed once per frame.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::air::Air;
use super::atmosphere::air;
pub use super::nearby::Nearby;
use super::observer::Observer;
use crate::space::Clock;
use crate::universe::{BodyId, GalaxyId, PlanetSpec};

/// The star system the sky is drawn for, empty outside a world.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct LocalSystem(pub Option<Nearby>);

impl LocalSystem {
    /// Returns the air of planet number `body`, or the home air outside a world.
    pub fn air(&self, body: usize) -> Air {
        let near = self.0.as_ref();
        let planet = near.and_then(|near| near.planet(u8::try_from(body).ok()?));
        planet.map_or(Air::HOME, |planet| Air::of(&planet.record))
    }

    /// Returns the air of the body under the camera of `vantage`, planet or moon, or the home
    /// air outside a world.
    pub fn air_below(&self, vantage: &Vantage) -> Air {
        if vantage.moon == 0 {
            return self.air(vantage.body);
        }
        let near = self.0.as_ref();
        let body = near.and_then(|near| near.body(u8::try_from(vantage.body).ok()?, vantage.moon));
        body.map_or(Air::HOME, |body| Air::of(&body.record))
    }
}

/// Where the camera is and what surrounds it, in the units the sky systems draw with.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Vantage {
    /// The camera in the star system.
    pub observer: Observer,
    /// Camera position in render space.
    pub camera: Vec3,
    /// Camera velocity in metres per second in render axes, against the ground or the star.
    pub velocity: Vec3,
    /// Unit vector away from the planet centre, in render axes.
    pub up: Vec3,
    /// Height of the camera above sea level in metres.
    pub altitude: f32,
    /// Share of a home atmosphere left above the camera: 1 at the ground of a planet with at
    /// least the home pressure, less in thinner air, 0 in space and on a planet without air.
    pub air: f32,
    /// Rank around the star of the planet the camera is on or near, or whose moon it is on.
    pub body: usize,
    /// Rank of the moon under the camera around that planet, 0 when it is the planet itself.
    pub moon: u8,
    /// Sea level radius of the planet in metres.
    pub radius: f32,
    /// Height above sea level in metres where the air ends.
    pub atmosphere: f32,
    /// Height above sea level in metres of the bottom of the cloud layer.
    pub cloud_base: f32,
    /// Height above sea level in metres of the top of the cloud layer.
    pub cloud_top: f32,
    /// Galaxy the camera is in.
    pub galaxy: GalaxyId,
    /// Camera position in metres from the galaxy centre.
    pub galactic: DVec3,
    /// How deep the camera sits in the star disc of its galaxy: 1 inside, 0 well outside.
    pub band: f32,
}

impl Default for Vantage {
    /// Stands at sea level under a full sky, the state before a world exists.
    fn default() -> Self {
        Self {
            observer: Observer::default(),
            camera: Vec3::ZERO,
            velocity: Vec3::ZERO,
            up: Vec3::Y,
            altitude: 0.0,
            air: 1.0,
            body: 0,
            moon: 0,
            radius: 1.0,
            atmosphere: 0.0,
            cloud_base: 0.0,
            cloud_top: 0.0,
            galaxy: GalaxyId::default(),
            galactic: DVec3::ZERO,
            band: 1.0,
        }
    }
}

impl Vantage {
    /// Builds the vantage of an observer near `planet` of the system `local`.
    pub fn of(observer: &Observer, local: &Nearby, planet: &PlanetSpec, camera: Vec3) -> Self {
        let altitude = observer.altitude(planet.radius) as f32;
        let galactic = local.system.star.pos + observer.eye;
        let out = (galactic.length() / local.galaxy_radius.max(1.0)) as f32;
        Self {
            observer: *observer,
            camera,
            velocity: Vec3::ZERO,
            up: (observer.to_render * observer.up()).as_vec3(),
            altitude,
            air: air(altitude, planet.atmosphere_height as f32) * Air::of(&planet.record).share(),
            body: usize::from(planet.id.index),
            moon: planet.id.moon,
            radius: planet.radius as f32,
            atmosphere: planet.atmosphere_height as f32,
            cloud_base: planet.cloud_base as f32,
            cloud_top: planet.cloud_top as f32,
            galaxy: local.id.galaxy,
            galactic,
            band: 1.0 - super::celestial::smoothstep(0.8, 1.5, out),
        }
    }

    /// Returns this vantage with the camera moving at `velocity` metres per second in render axes.
    pub fn moving(self, velocity: Vec3) -> Self {
        Self { velocity, ..self }
    }

    /// Returns true when `id` is the planet or moon under the camera.
    pub fn is(&self, id: BodyId) -> bool {
        usize::from(id.index) == self.body && id.moon == self.moon
    }

    /// Returns the unit vector in render axes from a point `pos` metres from the star toward the star.
    pub fn sunward(&self, pos: DVec3) -> Vec3 {
        (self.observer.to_render * (-pos).normalize_or(DVec3::X)).as_vec3()
    }
}

/// The current star system at the current universe time, for the systems that draw its bodies.
#[derive(SystemParam)]
pub struct Scene<'w> {
    /// Where the camera is.
    pub vantage: Res<'w, Vantage>,
    /// The star system around it.
    pub local: Res<'w, LocalSystem>,
    /// Universe time.
    pub clock: Res<'w, Clock>,
}

impl Scene<'_> {
    /// Returns a planet or moon of the current system with its position in metres from the star.
    pub fn body(&self, id: BodyId) -> Option<(&PlanetSpec, DVec3)> {
        let body = self.local.0.as_ref()?.find(id)?;
        Some((body, body.position(self.clock.seconds)))
    }
}
