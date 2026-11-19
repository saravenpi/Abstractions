//! Where a marker lies around the player for the compass: its bearing from the view and its
//! distance, in the frame of the eye of this frame.
//!
//! A marker on the planet the player stands on, or given in the anchor frame, is aimed at
//! directly. The home base away from its planet gives the next place on the way home. Any
//! other marker on another body is aimed at through the centre of that body plus its offset.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::super::eye::{Eye, HudPose};
use super::super::needle::bearing;
use crate::computer::{Target, homeward, ship_position, target_position};
use crate::menus::{Marker, MarkerKind, MarkerPlace};
use crate::space::{Clock, Frame, Location, Travel};
use crate::universe::{BodyId, Universe};
use crate::world::{Planet, spin};

/// Distance in metres under which a target is here and its mark is hidden.
const HERE: f64 = 12.0;

/// Distance in metres past which a favourite leaves the compass.
pub const FAVOURITE_REACH: f64 = 2000.0;

/// What the compass reads to aim at markers.
#[derive(SystemParam)]
pub struct Around<'w> {
    /// The eye of the HUD for this frame.
    eye: Res<'w, HudPose>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The universe and its time, which say where far places are.
    sky: Option<(Res<'w, Universe>, Res<'w, Clock>)>,
    /// The planet the player is on, whose spin turns the sky.
    planet: Option<Res<'w, Planet>>,
}

impl Around<'_> {
    /// Returns the bearing of `marker` from the view, in radians, and its distance in metres,
    /// or nothing when it has no place the compass can aim at, is right here, or is a mark
    /// the compass leaves out: the ship while seated, a favourite past its reach.
    pub fn reading(&self, marker: &Marker) -> Option<(f32, f64)> {
        let eye = self.eye.0.as_ref()?;
        let seated = self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip);
        if marker.kind == MarkerKind::Ship && seated {
            return None;
        }
        let (offset, gap) = self.target(eye, marker)?;
        let far = marker.kind == MarkerKind::Favourite && gap > FAVOURITE_REACH;
        let around = bearing(
            offset.as_vec3(),
            eye.forward.as_vec3(),
            above(eye).as_vec3(),
        )?;
        (gap > HERE && !far).then_some((around, gap))
    }

    /// Returns the bearing of the north pole of the planet under the player from the view, in
    /// radians, or nothing off a planet or right at a pole. North is up the spin axis, the
    /// axis the coordinates read the latitude along.
    pub fn north(&self) -> Option<f32> {
        let eye = self.eye.0.as_ref()?;
        let Frame::Surface(_) = eye.anchor.frame else {
            return None;
        };
        let offset = DVec3::Y * eye.at.length() - eye.at;
        let around = bearing(
            offset.as_vec3(),
            eye.forward.as_vec3(),
            above(eye).as_vec3(),
        )?;
        (offset.length() > HERE).then_some(around)
    }

    /// Returns the offset in metres from the eye to `marker`, in the axes of the eye frame, and
    /// its length.
    fn target(&self, eye: &Eye, marker: &Marker) -> Option<(DVec3, f64)> {
        let ground = match eye.anchor.frame {
            Frame::Surface(body) => Some(body),
            _ => None,
        };
        let at = match marker.place {
            MarkerPlace::Anchor(at) => at,
            MarkerPlace::Body(body, at) if Some(body) == ground => at,
            MarkerPlace::Body(..) if marker.kind == MarkerKind::Ship => return None,
            MarkerPlace::Body(..) if marker.kind == MarkerKind::Home => return self.far(eye),
            MarkerPlace::Body(body, at) => return self.remote(eye, body, at),
        };
        let offset = at - eye.at;
        Some((offset, offset.length()))
    }

    /// Returns the offset to the next place on the way home and its length.
    fn far(&self, eye: &Eye) -> Option<(DVec3, f64)> {
        let (universe, clock) = self.sky.as_ref()?;
        let (toward, gap) = homeward(universe, eye.anchor.frame, eye.at, clock.seconds)?;
        Some((self.turned(eye, toward * gap), gap))
    }

    /// Returns the offset to the point `at` of the body `body`, measured from its centre
    /// without its spin, which is close enough from another body, and its length.
    fn remote(&self, eye: &Eye, body: BodyId, at: DVec3) -> Option<(DVec3, f64)> {
        let (universe, clock) = self.sky.as_ref()?;
        let from = ship_position(universe, eye.anchor.frame, eye.at, clock.seconds)?;
        let to = target_position(universe, Target::Body(body), clock.seconds)? + at;
        let offset = to - from;
        Some((self.turned(eye, offset), offset.length()))
    }

    /// Returns `offset`, given in the axes of the systems, in the axes of the eye frame: on a
    /// planet the sky turns against its spin.
    fn turned(&self, eye: &Eye, offset: DVec3) -> DVec3 {
        let clock = self.sky.as_ref().map_or(0.0, |(_, clock)| clock.seconds);
        let planet = self.planet.as_deref();
        let day = planet.filter(|planet| Frame::Surface(planet.body) == eye.anchor.frame);
        let turn = day.map_or(DQuat::IDENTITY, |planet| spin(planet.day_seconds, clock));
        turn.inverse() * offset
    }
}

/// Returns the up of the place of `eye`, around which the compass reads its bearings.
fn above(eye: &Eye) -> DVec3 {
    match eye.anchor.frame {
        Frame::Surface(_) if eye.anchor.flat => eye.anchor.rotation.inverse() * DVec3::Y,
        Frame::Surface(_) => eye.at.normalize(),
        _ => eye.up,
    }
}
