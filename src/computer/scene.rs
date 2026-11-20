//! What the computer reads from the world to fill its screen.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::arrow::bearing;
use super::atlas::Atlas;
use super::course::course;
use super::detail::worlds;
use super::locate::{here, ship_position, target_position};
use super::route::{RouteStatus, Target};
use super::rows::{Line, View};
use super::tab::Tab;
use super::track::Known;
use crate::render::GameCamera;
use crate::ship::{Ship, ShipTelemetry};
use crate::space::{Anchor, Clock, Frame, Location, UniPos};
use crate::universe::{PlanetSpec, Universe};
use crate::world::{Planet, spin};

/// The world as the computer sees it. Only valid while a world is loaded.
#[derive(SystemParam)]
pub(super) struct Scene<'w, 's> {
    pub(super) universe: Res<'w, Universe>,
    atlas: Res<'w, Atlas>,
    known: Res<'w, Known>,
    location: Res<'w, Location>,
    anchor: Res<'w, Anchor>,
    pub(super) clock: Res<'w, Clock>,
    telemetry: Option<Res<'w, ShipTelemetry>>,
    planet: Option<Res<'w, Planet>>,
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    ships: Query<'w, 's, &'static Ship>,
}

impl Scene<'_, '_> {
    /// Returns where the ship is, taken from the camera that rides in it.
    pub(super) fn ship(&self) -> UniPos {
        let eye = self
            .camera
            .iter()
            .next()
            .map_or(Vec3::ZERO, |at| at.translation());
        let at = self.anchor.to_frame(eye);
        ship_position(&self.universe, self.anchor.frame, at, self.clock.seconds).unwrap_or_default()
    }

    /// Returns the clockwise turn from north in radians of the arrow of the ship on the charts:
    /// where its nose points, turned from the axes of a spinning planet to those of its system.
    pub(super) fn facing(&self) -> Option<f32> {
        let frame = self.anchor.frame;
        let ship = self.ships.iter().find(|ship| ship.frame == frame)?;
        let day = self
            .planet
            .as_deref()
            .filter(|planet| Frame::Surface(planet.body) == frame);
        let turn = day.map_or(DQuat::IDENTITY, |planet| {
            spin(planet.day_seconds, self.clock.seconds)
        });
        let rot = turn * ship.rot;
        Some(bearing(rot * DVec3::NEG_Z, rot * DVec3::Y))
    }

    /// Returns true on the frame the chart of known systems is finished.
    pub(super) fn charted(&self) -> bool {
        self.known.is_changed() && self.known.complete(&self.atlas)
    }

    /// Returns the heading and the rows of `tab`, on the chart of the star or the galaxy
    /// `inside`, or of the place of the ship.
    pub(super) fn table(&self, tab: Tab, inside: Option<Target>) -> (String, Vec<Line>) {
        let view = View {
            universe: &self.universe,
            atlas: &self.atlas,
            ship: self.ship(),
            time: self.clock.seconds,
        };
        let ship = self.location.body.star;
        let star = match inside {
            Some(Target::Star(star)) => star,
            _ => ship,
        };
        let galaxy = match inside {
            Some(Target::Galaxy(galaxy)) => galaxy,
            _ => star.galaxy,
        };
        match tab {
            Tab::System => (
                format!("{} system", Target::Star(star).name(&self.universe)),
                view.system(star),
            ),
            Tab::Galaxy => {
                let name = Target::Galaxy(galaxy).name(&self.universe);
                (
                    format!("{name} galaxy"),
                    view.galaxy(galaxy, self.known.stars()),
                )
            }
            Tab::Universe => ("Known universe".to_string(), view.universe()),
        }
    }

    /// Returns, for each of `lines`, the planet whose record the computer knows.
    pub(super) fn worlds(&self, lines: &[Line]) -> Vec<Option<PlanetSpec>> {
        worlds(&self.universe, &self.atlas, lines)
    }

    /// Returns the planet, the system and the galaxy the ship is inside.
    pub(super) fn here(&self) -> [Option<Target>; 3] {
        here(&self.universe, self.anchor.frame, self.anchor.origin)
    }

    /// Returns the course line: the destination, its distance, a rough time to get there at
    /// the speed of the ship, and the state of the route.
    pub(super) fn route(&self, destination: Option<Target>, status: RouteStatus) -> String {
        let name = destination.map(|target| target.name(&self.universe));
        let at = destination
            .and_then(|target| target_position(&self.universe, target, self.clock.seconds));
        let gap = at.map_or(0.0, |at| at.distance(self.ship()));
        let speed = self
            .telemetry
            .as_ref()
            .map_or(0.0, |ship| ship.orbital_speed);
        course(name.as_deref().map(|name| (name, gap)), speed, status)
    }
}
