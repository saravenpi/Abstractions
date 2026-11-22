//! Placing the ship: its render pose, the render anchor while flying and the planet the world streams.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::body::Ship;
use super::mooring::anchor_for;
use super::route::ARRIVAL;
use super::sim::{Cosmos, Pace};
use super::sky::Skies;
use super::tune::BODY_SWITCH;
use super::warp::Warp;
use super::wells::Well;
use crate::player::Player;
use crate::render::GameCamera;
use crate::space::{Anchor, Clock, Frame, Location, Placed, Rebased, Shift, Travel};
use crate::universe::{BodyId, PlanetSpec};
use crate::view::CameraPose;
use crate::world::enter_planet;

/// Returns where the ship is drawn: its position moved by the frame time not yet simulated.
fn shown(ship: &Ship, carry: f64) -> DVec3 {
    let still = ship.landed || matches!(ship.warp, Warp::Active { .. });
    ship.pos + ship.vel * if still { 0.0 } else { carry }
}

/// Filter of the free root entities of a planet, without the player body or the camera.
type Free = (
    With<Rebased>,
    Without<ChildOf>,
    Without<Player>,
    Without<GameCamera>,
);

/// Filter of a root game camera, not riding the player body.
type LooseCamera = (With<GameCamera>, Without<ChildOf>, Without<Player>);

/// What the anchor follows.
#[derive(SystemParam)]
pub struct Berth<'w> {
    /// Where the player is.
    location: Res<'w, Location>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// Bookkeeping of the fixed step.
    pace: Res<'w, Pace>,
    /// Universe time.
    clock: Res<'w, Clock>,
}

/// What the camera keeps in render space when the anchor moves.
#[derive(SystemParam)]
pub struct View<'w, 's> {
    /// The pose the camera rig wrote this frame, if the rig is active.
    pose: Option<ResMut<'w, CameraPose>>,
    /// A root game camera, not riding the player body.
    camera: Query<'w, 's, &'static mut Transform, LooseCamera>,
}

/// Returns the rigid shift taking the ship's render pose under `old` to its new pose under `new`,
/// used when the anchor changes frame and `Shift::between` does not apply.
fn frame_shift(old: &Anchor, new: &Anchor, placed: &Placed, (pos, rot): (DVec3, DQuat)) -> Shift {
    let old_rot = old.rotation_to_render(placed.rot);
    let new_rot = new.rotation_to_render(rot);
    let rotation = (new_rot * old_rot.inverse()).normalize();
    let translation = new.to_render(pos) - rotation * old.to_render(placed.pos);
    Shift {
        rotation,
        translation,
    }
}

/// Carries the camera pose and any root game camera across an anchor shift.
fn shift_view(shift: &Shift, view: &mut View) {
    if let Some(pose) = view.pose.as_deref_mut().filter(|pose| pose.fov > 0.0) {
        pose.position = shift.point(pose.position);
        pose.rotation = shift.turn(pose.rotation);
    }
    for mut transform in &mut view.camera {
        transform.translation = shift.point(transform.translation);
        transform.rotation = shift.turn(transform.rotation);
    }
}

/// Keeps the render anchor at the ship while the player is seated, and carries the free
/// entities of a planet and the camera pose along when the anchor moves. The seated body is
/// placed again after it, under the new anchor.
///
/// It runs at the end of the frame, before the poses are copied and propagated: every system of
/// `Update` reads one anchor with the transforms and global transforms made under it. Moved in
/// `Update`, it left the systems after it reading the new anchor against transforms of the old,
/// off by a whole rebase for a frame, which made the ground, the sky and the map flash.
pub fn moor(
    berth: Berth,
    mut anchor: ResMut<Anchor>,
    ship: Single<(&Ship, &Placed)>,
    mut free: Query<&mut Transform, Free>,
    mut view: View,
) {
    if berth.location.travel != Travel::InShip || anchor.flat {
        return;
    }
    let (ship, placed) = *ship;
    let at = (ship.frame, shown(ship, berth.pace.carry));
    let next = anchor_for(&anchor, at, berth.skies.planet(), berth.clock.seconds);
    if next == *anchor {
        return;
    }
    let shift = if next.frame == anchor.frame {
        Shift::between(&anchor, &next)
    } else {
        frame_shift(&anchor, &next, placed, (at.1, ship.rot))
    };
    if next.frame == anchor.frame && matches!(next.frame, Frame::Surface(_)) {
        for mut transform in &mut free {
            transform.translation = shift.point(transform.translation);
            transform.rotation = shift.turn(transform.rotation);
        }
    }
    shift_view(&shift, &mut view);
    *anchor = next;
}

/// Writes the pose of the ship in the anchor frame, which the frame systems copy to its transform.
pub fn pose(anchor: Res<Anchor>, pace: Res<Pace>, ship: Single<(&Ship, &mut Placed)>) {
    let (ship, mut placed) = ship.into_inner();
    let next = Placed {
        pos: shown(ship, pace.carry),
        rot: ship.rot,
    };
    if anchor.frame == ship.frame && *placed != next {
        *placed = next;
    }
}

/// Returns the planet the ship is at: the one whose surface frame it is in, or in the frame of
/// a star the nearest planet once it is within [`BODY_SWITCH`] of its Hill radius or within the
/// distance the autopilot drops out at, which a planet close to its star reaches long before the share
/// of its small Hill sphere, or `current` is not of that star.
fn body_at(ship: &Ship, skies: &Skies, current: BodyId) -> Option<BodyId> {
    let star = match ship.frame {
        Frame::Surface(body) => return Some(body),
        Frame::System(star) => star,
        Frame::Galaxy(_) => return None,
    };
    let system = skies.spec().filter(|system| system.star.id == star)?;
    let score = |(planet, well): (&PlanetSpec, &Well)| {
        let docked = well.pos.distance(ship.pos) <= well.lock * ARRIVAL;
        (well.reach(ship.pos), planet.id, docked)
    };
    let scored = system.planets.iter().zip(skies.wells.get(1..)?).map(score);
    let (reach, body, docked) = scored.min_by(|a, b| a.0.total_cmp(&b.0))?;
    (reach < BODY_SWITCH || docked || current.star != star).then_some(body)
}

/// Names the planet the seated player is at and has the world stream it.
pub fn track(
    mut commands: Commands,
    cosmos: Cosmos,
    skies: Res<Skies>,
    mut location: ResMut<Location>,
    ship: Single<&Ship>,
) {
    if location.travel != Travel::InShip {
        return;
    }
    if let Some(body) = body_at(&ship, &skies, location.body).filter(|body| *body != location.body)
    {
        location.body = body;
    }
    if cosmos
        .planet
        .as_ref()
        .is_some_and(|planet| planet.body != location.body)
    {
        enter_planet(&mut commands, &cosmos.universe, location.body);
    }
}
