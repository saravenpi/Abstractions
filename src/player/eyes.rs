//! The eyes of the body: what the camera rig follows on foot and in flight.

use bevy::prelude::*;

use super::chase::{carried, chased, drift};
use super::controller::Motion;
use super::upright::tilt;
use super::{Player, PlayerSet, local};
use crate::gear::{FlightState, Gear};
use crate::screens::{in_control, playing};
use crate::sim::PlayerId;
use crate::space::{Anchor, on_foot, up};
use crate::view::{Heading, Kind, Look, Target, ViewSet};

pub use super::chase::WingView;

/// The mouse button that lets the view steer a body on its wings while it is held.
pub(super) const STEER: MouseButton = MouseButton::Left;

/// Airspeed in metres per second above which a body the gear carries is framed as flying even
/// with the wings folded: a fast fall.
const SWIFT: f32 = 18.0;

/// Returns how the camera frames a body whose gear is in `state`: as flying with the wings
/// out or in a fast fall, else as on foot. The rig blends between the two.
pub(super) fn kind(state: &FlightState) -> Kind {
    let swift = state.mode.airborne() && state.airspeed > SWIFT;
    match state.mode.winged() || swift {
        true => Kind::Flying,
        false => Kind::OnFoot,
    }
}

/// Turns the view of the local body on its wings: a chase view.
///
/// The mouse looks around freely. With the steer button up the view also turns with the body,
/// by as much as its nose heading and pitch turned since the last frame, over the top of a loop
/// too, and once left alone it drifts back behind the body. A view on its back behind a body
/// that faces the sky rights itself at once, see `chased`. With the button held the view is
/// the aim of the flight, so it turns with the mouse alone: carried with the body as well, it
/// would chase its own turn.
pub(super) fn steer(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    players: Query<(&PlayerId, &Gear), With<Player>>,
    (heading, mut look): (Res<Heading>, ResMut<Look>),
    mut wing: ResMut<WingView>,
) {
    let Some(gear) = local(players.iter().map(|(id, gear)| (*id, gear))) else {
        return;
    };
    let state = &gear.state;
    if !state.mode.winged() {
        *wing = WingView::default();
        return;
    }
    let (dt, held) = (time.delta_secs(), buttons.pressed(STEER));
    let body = Vec2::new(state.servo.yaw, state.pitch);
    let (before, mouse) = (
        Vec2::new(heading.yaw, heading.pitch),
        Vec2::new(look.yaw_delta, look.pitch_delta),
    );
    let carry = carried(&mut wing, body, before) * f32::from(u8::from(!held));
    wing.idle = match held || mouse != Vec2::ZERO {
        true => 0.0,
        false => wing.idle + dt,
    };
    let view = before + carry;
    let goal = chased(&mut wing, state, body, view);
    let turn = mouse + carry + drift(goal, view, &wing, dt) * f32::from(u8::from(!held));
    (look.yaw_delta, look.pitch_delta) = (turn.x, turn.y);
}

/// The parts of a player body the camera rig follows.
type Followed = (
    &'static PlayerId,
    &'static Transform,
    &'static Motion,
    &'static Gear,
);

/// Tells the camera rig what it follows: the local body, upright, with the velocity the controller
/// just gave it, shifted by how far the eyes still trail it after a step-up. The forward
/// direction is the north of the upright frame, because the rig owns the heading of the view.
/// In the air it also gives the way the back of the body faces, so the view rolls over with
/// it a full turn round, however the gear reads the attitude.
pub(super) fn aim(
    anchor: Res<Anchor>,
    players: Query<Followed, With<Player>>,
    mut target: ResMut<Target>,
) {
    let followed = players
        .iter()
        .map(|(id, transform, motion, gear)| (*id, (transform, motion, gear)));
    let Some((transform, motion, gear)) = local(followed) else {
        return;
    };
    let vertical = up(&anchor, transform.translation);
    let to_render = tilt(vertical);
    let airborne = gear.state.mode.airborne();
    *target = Target {
        position: transform.translation - to_render * motion.lag,
        up: vertical,
        forward: to_render * Vec3::NEG_Z,
        velocity: to_render * motion.velocity,
        kind: kind(&gear.state),
        back: gear.state.frame() * Vec3::Y * f32::from(u8::from(airborne)),
        ..Target::default()
    };
}

/// Registers the chase view of the wings, before the intents are sent, and the feed of the
/// camera rig, after the controller moved the body and before the rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Target>()
        .init_resource::<WingView>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_systems(
            Update,
            (
                steer
                    .after(ViewSet::Input)
                    .before(PlayerSet::Intend)
                    .run_if(in_control),
                aim.after(PlayerSet::Move).before(ViewSet::Rig),
            )
                .run_if(playing)
                .run_if(on_foot),
        );
}
