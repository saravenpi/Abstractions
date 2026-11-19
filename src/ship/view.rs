//! The ship as the camera rig sees it: the seat the body sits in, the target the rig follows,
//! and the shake of rough air on top of the view the rig made.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::shake::{Buffet, JOLT, shake};
use super::telemetry::{ShipTelemetry, WarpState};
use crate::player::Player;
use crate::render::{GameCamera, smoothstep};
use crate::space::{Anchor, Frame, Location, Placed, Travel, up};
use crate::view::{Kind, Target};

/// Eye point of the pilot in body axes, under the canopy. The body of the player sits there.
pub const SEAT: DVec3 = DVec3::new(0.0, 0.9, -1.3);

/// Radius of the sphere that holds the whole hull, in metres.
pub const HULL_RADIUS: f32 = 4.5;

/// Heights above the ground in metres between which the chase camera goes from trailing
/// level with the ground to trailing the ship freely.
const GROUND_BAND: (f32, f32) = (150.0, 2500.0);

/// Run condition: a world exists and the player is seated in the ship.
pub fn seated(location: Option<Res<Location>>) -> bool {
    location.is_some_and(|location| location.travel == Travel::InShip)
}

/// Filter of the player body, which is not the camera.
type OnlyBody = (With<Player>, Without<GameCamera>);

/// What the seat and the target are read from.
#[derive(SystemParam)]
pub struct Cockpit<'w> {
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// The instruments.
    telemetry: Res<'w, ShipTelemetry>,
}

impl Cockpit<'_> {
    /// Returns how near a surface the ship flies, as the local up scaled from 1 low in an
    /// atmosphere to 0 high above it or in open space.
    fn ground(&self, at: Vec3) -> Vec3 {
        let Frame::Surface(_) = self.anchor.frame else {
            return Vec3::ZERO;
        };
        let height = self.telemetry.altitude_ground as f32;
        up(&self.anchor, at) * (1.0 - smoothstep(GROUND_BAND.0, GROUND_BAND.1, height))
    }

    /// Returns the velocity of the ship relative to its surroundings in render axes, or the
    /// warp speed along its nose during a warp.
    fn velocity(&self, ship: &Ship, nose: Vec3) -> Vec3 {
        if let WarpState::Active(speed) = self.telemetry.warp {
            return nose * speed as f32;
        }
        let along = (self.anchor.rotation * ship.vel)
            .as_vec3()
            .normalize_or_zero();
        along * self.telemetry.surface_speed as f32
    }
}

/// Seats the player body in the pilot seat, turned with the hull, and tells the camera rig to
/// follow the ship. The rig places the camera; the body only carries it.
pub fn carry(
    cockpit: Cockpit,
    ship: Single<(&Ship, &Placed)>,
    mut body: Single<&mut Transform, OnlyBody>,
    mut target: ResMut<Target>,
) {
    let (ship, placed) = *ship;
    let anchor = &*cockpit.anchor;
    let rot = anchor.rotation_to_render(placed.rot);
    let centre = anchor.to_render(placed.pos);
    body.translation = anchor.to_render(placed.pos + placed.rot * SEAT);
    body.rotation = rot;
    let spin = (anchor.rotation * placed.rot * ship.spin).as_vec3();
    *target = Target {
        position: centre,
        up: rot * Vec3::Y,
        forward: rot * Vec3::NEG_Z,
        velocity: cockpit.velocity(ship, rot * Vec3::NEG_Z),
        spin,
        ground: cockpit.ground(centre),
        radius: HULL_RADIUS,
        kind: Kind::Ship,
        ..Target::default()
    };
}

/// Shakes the view the rig made in rough air and after a crash: an offset and a tilt on top of it. The rig
/// writes the whole camera pose every frame, so the shake never adds up.
pub fn buffet(
    time: Res<Time>,
    level: Res<Buffet>,
    mut eyes: Single<&mut Transform, With<GameCamera>>,
) {
    let strength = level.level + level.jolt * JOLT.0;
    if strength <= 0.0 {
        return;
    }
    let (offset, tilt) = shake(time.elapsed_secs_wrapped(), strength);
    let turned = eyes.rotation * offset;
    eyes.translation += turned;
    eyes.rotation = (eyes.rotation * tilt).normalize();
}
