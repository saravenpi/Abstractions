//! What the controller cannot tell the body by itself: the flight gear, diving, the look of
//! the head, and how much of the body the camera shows.
//!
//! It runs between [`super::AvatarSet::Fill`] and [`super::AvatarSet::Animate`]. The attitude
//! of the gear only poses the body: the trunk lies prone and banks inside the player entity,
//! which itself stays upright, so neither the physics body nor the camera ever rolls.

use std::f32::consts::PI;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{AvatarPose, Mode, Shown};
use crate::gear::{FlightState, Gear, Mode as Flight};
use crate::player::{Motion, Player, wrap};
use crate::space::{Anchor, Location, Travel, altitude};
use crate::view::{BodyFade, Heading};
use crate::world::Terrain;

/// Height of the body centre above the sea under which a swimming body dives, in metres.
const DIVE_BELOW: f32 = -1.2;

/// Share of the body fade above which the whole body is drawn, head and screen included.
const SHOW_HEAD: f32 = 0.5;

/// How hard a landing roll looks at least, from 0 to 1.
const ROLL_HARD: f32 = 0.7;

/// Returns the mode of a body the filler put in `filled`, whose gear is in `state`, `deep`
/// under the sea or not.
pub fn mode(filled: Mode, state: &FlightState, deep: bool) -> Mode {
    match state.mode {
        Flight::BootFall => Mode::BootFall,
        Flight::Glide => Mode::Glide,
        Flight::Fly => Mode::Fly,
        Flight::Fall if state.velocity.y > 0.5 => Mode::Jump,
        Flight::Fall => Mode::Fall,
        Flight::Roll => Mode::Land,
        Flight::Slide | Flight::Off if filled == Mode::Swim && deep => Mode::Dive,
        Flight::Slide | Flight::Off => filled,
    }
}

/// Writes the gear, the dive and the look of the head over the filled `pose`.
///
/// `look` is where the view points relative to the body: yaw to the left, then pitch up.
pub fn dress(pose: &mut AvatarPose, state: &FlightState, look: Vec2, deep: bool) {
    let airborne = state.mode.airborne();
    pose.mode = mode(pose.mode, state, deep);
    pose.bank = if airborne { state.bank } else { 0.0 };
    pose.pitch = match (airborne, state.pitch.abs() > PI) {
        (false, _) => 0.0,
        (true, false) => state.pitch,
        (true, true) => wrap(state.pitch),
    };
    pose.thrust = state.power();
    pose.wings = state.spread();
    if airborne {
        pose.airspeed = state.airspeed;
    }
    if state.mode == Flight::Roll {
        pose.landed_hard = pose.landed_hard.max(ROLL_HARD);
    }
    pose.look = look;
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
    /// The player body.
    player: Single<'w, 's, (&'static Transform, &'static Motion, &'static Gear), With<Player>>,
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

/// Dresses the pose input from the gear and the view, and chooses what is drawn.
pub fn wear(worn: Worn, mut pose: ResMut<AvatarPose>, mut drawn: ResMut<Shown>) {
    let (transform, motion, gear) = *worn.player;
    let seated = worn
        .location
        .is_some_and(|location| location.travel == Travel::InShip);
    let height = altitude(&worn.anchor, worn.terrain.globe(), transform.translation);
    let look = match seated {
        true => Vec2::ZERO,
        false => Vec2::new(wrap(worn.heading.yaw - motion.yaw), worn.heading.pitch),
    };
    dress(&mut pose, &gear.state, look, height < DIVE_BELOW);
    drawn.set_if_neq(shown(seated, worn.fade.0));
}
