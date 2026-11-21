//! What the controller cannot tell the body by itself: the flight gear, diving, the look of
//! the head, and how much of the body the camera shows.
//!
//! It runs between [`super::AvatarSet::Fill`] and [`super::AvatarSet::Animate`]. The attitude
//! of the gear only poses the body: the trunk lies prone and banks inside the player entity,
//! which itself stays upright, so neither the physics body nor the camera ever rolls.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::air::level;
use super::fill::hardness;
use super::own::Own;
use super::touch::{Touch, touch};
use super::{AvatarPose, Mode, Shown};
use crate::gear::{FlightState, Gear, Mode as Flight};
use crate::player::{Motion, Player, above_sea, wrap};
use crate::space::{Anchor, Location, Travel};
use crate::view::{BodyFade, Heading};
use crate::world::Terrain;

/// Height of the body centre above the liquid surface under which a swimming body dives, in metres.
const DIVE_BELOW: f32 = -1.2;

/// Share of the body fade above which the whole body is drawn, head and screen included.
const SHOW_HEAD: f32 = 0.5;

/// How hard a landing roll looks at least, from 0 to 1.
const ROLL_HARD: f32 = 0.7;

/// Returns the mode of a body the filler put in `filled`, whose gear is in `state` and takes a
/// touchdown as `touched`, `deep` under the sea or not.
pub fn mode(filled: Mode, state: &FlightState, touched: Touch, deep: bool) -> Mode {
    match state.mode {
        Flight::BootFall => Mode::BootFall,
        Flight::Glide => Mode::Glide,
        Flight::Fly => Mode::Fly,
        Flight::Fall if state.velocity.y > 0.5 => Mode::Jump,
        Flight::Fall => Mode::Fall,
        Flight::Roll | Flight::Slide => touched.mode(),
        Flight::Off if filled == Mode::Swim && deep => Mode::Dive,
        Flight::Off => filled,
    }
}

/// Keeps in `pose` how the body came in when the gear hit a surface this frame: the velocity
/// it had, turned into the frame of the body, and the speed it lost into the surface.
fn strike(pose: &mut AvatarPose, state: &FlightState) {
    if state.mode.airborne() && state.servo.impact > 0.0 {
        pose.struck = Quat::from_rotation_y(-state.servo.yaw) * state.servo.struck;
        pose.lost = state.servo.impact;
    }
}

/// Writes the gear, the dive, the way the body takes a touchdown and the look of the head over
/// the filled `pose`.
///
/// `look` is where the view points relative to the body: yaw to the left, then pitch up.
pub fn dress(pose: &mut AvatarPose, state: &FlightState, look: Vec2, deep: bool) {
    let airborne = state.mode.airborne();
    strike(pose, state);
    let touched = touch(state.mode, pose.struck, pose.lost, pose.landed_hard);
    pose.mode = mode(pose.mode, state, touched, deep);
    pose.bank = if airborne { state.bank } else { 0.0 };
    pose.pitch = if airborne { level(state.pitch) } else { 0.0 };
    pose.thrust = state.power();
    pose.wings = state.spread();
    pose.prone = state.prone;
    pose.load = if airborne { state.load } else { 1.0 };
    if airborne {
        pose.airspeed = state.airspeed;
    }
    if state.mode == Flight::Roll {
        pose.landed_hard = pose.landed_hard.max(ROLL_HARD).max(hardness(pose.lost));
    }
    pose.look = look;
}

/// Returns where the view of `heading` points relative to a body turned to `yaw`. The neck of
/// the body decides how much of it the head follows.
pub fn looked(heading: &Heading, yaw: f32) -> Vec2 {
    Vec2::new(wrap(heading.yaw - yaw), level(heading.pitch))
}

/// Returns how much of the body to draw: nothing seated in the ship, whose hull would show
/// it, all of it when the camera stands off, and all but the head at the eyes.
pub fn shown(seated: bool, fade: f32) -> Shown {
    match (seated, fade > SHOW_HEAD) {
        (true, _) => Shown::Hidden,
        (false, true) => Shown::Full,
        (false, false) => Shown::FirstPerson,
    }
}

/// What the dresser reads.
#[derive(SystemParam)]
pub struct Worn<'w, 's> {
    /// Which body is the one of the player at this machine.
    own: Own<'w, 's>,
    /// The player bodies.
    players: Query<'w, 's, (&'static Transform, &'static Motion, &'static Gear), With<Player>>,
    /// The look angles of the camera rig.
    heading: Res<'w, Heading>,
    /// How much of the body the camera wants drawn.
    fade: Res<'w, BodyFade>,
    /// Where the player is.
    location: Option<Res<'w, Location>>,
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// The terrain of the planet.
    terrain: Res<'w, Terrain>,
}

/// Returns the attitude of the gear in `state` in the frame of a player entity turned to `yaw`,
/// and keeps it in `held` while the gear carries the body through the air. On the ground it is
/// the one held from the last moment in the air, the attitude the flight poses blend out of
/// as the body lands, whichever way the entity turns meanwhile.
pub fn attitude(state: &FlightState, yaw: f32, held: &mut Quat) -> Quat {
    if state.mode.airborne() && state.attitude.is_finite() {
        *held = state.attitude;
    }
    Quat::from_rotation_y(-yaw) * *held
}

/// Dresses the pose input of the body at this machine from its gear and the view, and chooses
/// what is drawn of it.
pub fn wear(
    worn: Worn,
    mut poses: Query<&mut AvatarPose>,
    mut drawn: ResMut<Shown>,
    mut held: Local<Quat>,
) {
    let Some((player, body)) = worn.own.body() else {
        return;
    };
    let (Ok((transform, motion, gear)), Ok(mut pose)) =
        (worn.players.get(player), poses.get_mut(body))
    else {
        return;
    };
    let seated = worn
        .location
        .as_deref()
        .is_some_and(|location| location.travel == Travel::InShip);
    let height = above_sea(&worn.anchor, worn.terrain.globe(), transform.translation);
    let look = match seated {
        true => Vec2::ZERO,
        false => looked(&worn.heading, motion.yaw),
    };
    dress(&mut pose, &gear.state, look, height < DIVE_BELOW);
    pose.attitude = Some(attitude(&gear.state, motion.yaw, &mut held));
    pose.clearance = match motion.grounded {
        true => 0.0,
        false => gear.input.clearance,
    };
    drawn.set_if_neq(shown(seated, worn.fade.0));
}
