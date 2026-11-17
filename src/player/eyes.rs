//! The eyes of the body: what the camera rig follows on foot and in flight.

use bevy::prelude::*;

use super::controller::Motion;
use super::intent::axis;
use super::upright::tilt;
use super::{Player, PlayerSet};
use crate::gear::{FlightState, Gear};
use crate::screens::{in_control, playing};
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, on_foot, up};
use crate::view::{Kind, Look, Target, ViewSet};

/// Turn of the view in radians per second a held key gives a body on its wings: around the
/// local up, and up or down.
const WING_TURN: Vec2 = Vec2::new(1.3, 0.9);

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

/// Lets the movement keys steer a body on its wings. The wings follow the view, so the keys
/// turn the view the way the mouse does: left and right turn it, forward dives and back climbs.
/// Without this the keys only lean the body and the flight never leaves the line of the view.
fn steer(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Persistent<Settings>>,
    gear: Single<&Gear, With<Player>>,
    mut look: ResMut<Look>,
) {
    if !gear.state.mode.winged() {
        return;
    }
    let map = &settings.controls.keys;
    let turn = Vec2::new(
        axis(&keys, map.left, map.right),
        axis(&keys, map.back, map.forward),
    ) * WING_TURN
        * time.delta_secs();
    look.yaw_delta += turn.x;
    look.pitch_delta += turn.y;
}

/// Tells the camera rig what it follows: the body, upright, with the velocity the controller
/// just gave it. The forward direction is the north of the upright frame, because the rig
/// owns the heading of the view.
fn aim(
    anchor: Res<Anchor>,
    player: Single<(&Transform, &Motion, &Gear), With<Player>>,
    mut target: ResMut<Target>,
) {
    let (transform, motion, gear) = *player;
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

/// Registers the steering of the wings by the keys, before the controller, and the feed of
/// the camera rig, after the controller moved the body and before the rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Target>().add_systems(
        Update,
        (
            steer
                .after(ViewSet::Input)
                .before(PlayerSet::Move)
                .run_if(in_control),
            aim.after(PlayerSet::Move).before(ViewSet::Rig),
        )
            .run_if(playing)
            .run_if(on_foot),
    );
}
