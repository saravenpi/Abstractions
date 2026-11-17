//! What the computer reads from the world to fill its screen.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::atlas::Atlas;
use super::board::Tab;
use super::format::distance;
use super::locate::{ship_position, target_position};
use super::route::{RouteStatus, Target};
use super::rows::{Line, View};
use super::track::Known;
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Location, UniPos};
use crate::universe::Universe;

/// The world as the computer sees it. Only valid while a world is loaded.
#[derive(SystemParam)]
pub(super) struct Scene<'w, 's> {
    universe: Res<'w, Universe>,
    atlas: Res<'w, Atlas>,
    known: Res<'w, Known>,
    location: Res<'w, Location>,
    anchor: Res<'w, Anchor>,
    clock: Res<'w, Clock>,
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
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

    /// Returns true on the frame the chart of known systems is finished.
    pub(super) fn charted(&self) -> bool {
        self.known.is_changed() && self.known.complete(&self.atlas)
    }

    /// Returns the heading and the rows of `tab`.
    pub(super) fn table(&self, tab: Tab) -> (String, Vec<Line>) {
        let view = View {
            universe: &self.universe,
            atlas: &self.atlas,
            ship: self.ship(),
            time: self.clock.seconds,
        };
        let star = self.location.body.star;
        match tab {
            Tab::System => (
                format!("{} system", Target::Star(star).name(&self.universe)),
                view.system(star),
            ),
            Tab::Galaxy => {
                let name = Target::Galaxy(star.galaxy).name(&self.universe);
                (
                    format!("{name} galaxy"),
                    view.galaxy(star.galaxy, self.known.stars()),
                )
            }
            Tab::Universe => ("Known universe".to_string(), view.universe()),
        }
    }

    /// Returns the line that names the destination, its distance and the state of the route.
    pub(super) fn route(&self, destination: Option<Target>, status: RouteStatus) -> String {
        let Some(target) = destination else {
            return format!("No destination - route {}", status.label());
        };
        let name = target.name(&self.universe);
        let gap = target_position(&self.universe, target, self.clock.seconds)
            .map(|at| distance(at.distance(self.ship())))
            .unwrap_or_default();
        format!("Destination {name}, {gap} - route {}", status.label())
    }
}
