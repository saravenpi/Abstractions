//! A place on the streamed planet seen from on it or above it: where it is in the frame of the
//! eye, whether the planet stands between them, and how far it is over the ground. The home base
//! on the far side of its planet is still aimed at, through the planet, and is shown as hidden.

use bevy::math::DVec3;

use super::super::eye::Eye;
use super::aim::Around;
use crate::menus::{Marker, MarkerPlace};
use crate::universe::BodyId;
use crate::world::{Link, Planet};

/// Metres under a place and under the eye that the sphere the planet hides things with keeps:
/// the ground a place stands on does not hide it.
const SINK: f64 = 100.0;

/// Returns whether the line from `eye` to `target` passes through the sphere of `radius` metres
/// around `centre`.
pub(super) fn behind(eye: DVec3, target: DVec3, (centre, radius): (DVec3, f64)) -> bool {
    let span = target - eye;
    let long = span.length_squared();
    if long <= 0.0 {
        return false;
    }
    let share = ((centre - eye).dot(span) / long).clamp(0.0, 1.0);
    (eye + span * share).distance(centre) < radius
}

impl Around<'_> {
    /// Returns the link from the streamed planet `body` to the frame of `eye`, with the planet,
    /// or nothing while another planet is streamed, the eye is out of its system or the world is
    /// the flat plane.
    fn link(&self, eye: &Eye, body: BodyId) -> Option<(Link, &Planet)> {
        let (universe, clock) = self.sky.as_ref()?;
        let planet = self
            .planet
            .as_deref()
            .filter(|planet| planet.body == body)?;
        let link = Link::between(eye.anchor.frame, planet, universe, clock.seconds)?;
        (!eye.anchor.flat).then_some((link, planet))
    }

    /// Returns the offset in metres from `eye` to the point `at` of the streamed planet `body`,
    /// in the axes of the eye frame, and its length.
    pub(super) fn orbiting(&self, eye: &Eye, body: BodyId, at: DVec3) -> Option<(DVec3, f64)> {
        let (link, _) = self.link(eye, body)?;
        let offset = link.to_anchor(at) - eye.at;
        Some((offset, offset.length()))
    }

    /// Returns whether the planet `marker` stands on hides it from the eye: the line of sight
    /// goes through the planet.
    pub fn hidden(&self, marker: &Marker) -> bool {
        let (Some(eye), MarkerPlace::Body(body, at)) = (self.eye.0.as_ref(), marker.place) else {
            return false;
        };
        let Some((link, planet)) = self.link(eye, body) else {
            return false;
        };
        let lowest = (at.length() - SINK).min(eye.at.distance(link.offset) - SINK);
        let radius = planet.globe.radius().min(lowest);
        behind(eye.at, link.to_anchor(at), (link.offset, radius))
    }

    /// Returns the metres to `marker` over the ground of its planet: along the ground to the point
    /// under it, with the height between the two on top, or nothing off that planet.
    pub fn over_ground(&self, marker: &Marker) -> Option<f64> {
        let (Some(eye), MarkerPlace::Body(body, at)) = (self.eye.0.as_ref(), marker.place) else {
            return None;
        };
        let (link, planet) = self.link(eye, body)?;
        let from = eye.at - link.offset;
        let to = link.to_anchor(at) - link.offset;
        let arc = from.angle_between(to) * planet.globe.radius();
        Some(arc.hypot(from.length() - to.length()))
    }
}
