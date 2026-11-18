//! The eyes of the body: what the camera rig follows on foot and in flight.

use bevy::prelude::*;

use super::controller::Motion;
use super::upright::tilt;
use super::{Player, PlayerSet, local, wrap};
use crate::gear::{FlightState, Gear};
use crate::screens::{in_control, playing};
use crate::sim::PlayerId;
use crate::space::{Anchor, on_foot, up};
use crate::view::{Heading, Kind, Look, Target, ViewSet};

/// Seconds the wings must have been open and the mouse left alone before the view drifts back
/// behind the body: long enough to look around, and to see where the wings opened.
const SETTLE: f32 = 2.5;

/// Rate per second at which the view drifts back behind a body on its wings.
const RETURN: f32 = 4.0;

/// Fastest the view drifts back, in radians per second: a view left far round swings back
/// steadily instead of whipping a third person camera across.
const SWING: f32 = 2.5;

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

/// Lets the view of a body on its wings look around freely and drift back behind it.
///
/// The keys steer the body itself and the mouse steers nothing, so the view is only a way of
/// looking. Once the wings have been open and the mouse left alone for [`SETTLE`] the view
/// eases back to the nose heading and the pitch of the flight frame, all the way round a
/// loop, at [`RETURN`] and never faster than [`SWING`].
fn follow(
    time: Res<Time>,
    players: Query<(&PlayerId, &Gear), With<Player>>,
    heading: Res<Heading>,
    mut look: ResMut<Look>,
    mut idle: Local<f32>,
) {
    let Some(gear) = local(players.iter().map(|(id, gear)| (*id, gear))) else {
        return;
    };
    let dt = time.delta_secs();
    let moved = look.yaw_delta != 0.0 || look.pitch_delta != 0.0;
    *idle = match moved || !gear.state.mode.winged() {
        true => 0.0,
        false => *idle + dt,
    };
    if *idle < SETTLE {
        return;
    }
    let state = &gear.state;
    let gap = Vec2::new(
        wrap(state.servo.yaw - heading.yaw),
        wrap(state.pitch - heading.pitch),
    );
    let turn = (gap * (1.0 - (-RETURN * dt).exp())).clamp_length_max(SWING * dt);
    look.yaw_delta += turn.x;
    look.pitch_delta += turn.y;
}

/// The parts of a player body the camera rig follows.
type Followed = (
    &'static PlayerId,
    &'static Transform,
    &'static Motion,
    &'static Gear,
);

/// Tells the camera rig what it follows: the local body, upright, with the velocity the controller
/// just gave it. The forward direction is the north of the upright frame, because the rig
/// owns the heading of the view.
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
        position: transform.translation,
        up: vertical,
        forward: to_render * Vec3::NEG_Z,
        velocity: to_render * motion.velocity,
        kind: kind(&gear.state),
        bank: gear.state.bank * f32::from(u8::from(airborne)),
        ..Target::default()
    };
}

/// Registers the drift of the view behind the wings, before the intents are sent, and the feed of
/// the camera rig, after the controller moved the body and before the rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Target>().add_systems(
        Update,
        (
            follow
                .after(ViewSet::Input)
                .before(PlayerSet::Intend)
                .run_if(in_control),
            aim.after(PlayerSet::Move).before(ViewSet::Rig),
        )
            .run_if(playing)
            .run_if(on_foot),
    );
}
