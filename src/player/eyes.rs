//! The eyes of the body: what the camera rig follows on foot and in flight.

use bevy::prelude::*;

use super::controller::Motion;
use super::intent::axis;
use super::upright::tilt;
use super::{Player, PlayerSet, wrap};
use crate::gear::{FlightState, Gear};
use crate::screens::{in_control, playing};
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, on_foot, up};
use crate::view::{Heading, Kind, Look, Target, ViewSet};

/// Turn of the view in radians per second a held key gives a body on its wings: around the
/// local up, and up or down.
const WING_TURN: Vec2 = Vec2::new(1.2, 0.9);

/// Farthest the view may pitch away from the nose of a body on its wings, in radians. The
/// nose follows the view only as fast as the wings can turn the flight, so a view that ran
/// ahead would come round to the other side of the nose and call it back the short way: this
/// leash is what lets a held key pull the body through a whole loop.
const LEASH: f32 = 0.9;

/// Share of that turn the roll keys give: they bank the body, and the view follows the banked turn.
const BANKED: f32 = 0.7;

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

/// Lets the keys steer a body on its wings. The wings follow the view, so the keys turn the
/// view the way the mouse does: the turn keys of the ship turn it, the left and right keys
/// roll the body into a banked turn that the view follows, forward dives and back climbs, as
/// far round as a loop, and neither the mouse nor the keys take the view farther than
/// [`LEASH`] from the nose; a view that is already farther is left where it is. Without this
/// the keys only lean the body and the flight never leaves the line of the view.
fn steer(
    time: Res<Time>,
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    gear: Single<&Gear, With<Player>>,
    heading: Res<Heading>,
    mut look: ResMut<Look>,
) {
    if !gear.state.mode.winged() {
        return;
    }
    let map = &settings.controls.keys;
    let flat = axis(&keys, map.ship.turn_left, map.ship.turn_right);
    let banked = axis(&keys, map.left, map.right) * BANKED;
    let turn = Vec2::new(
        (flat + banked).clamp(-1.0, 1.0),
        axis(&keys, map.back, map.forward),
    ) * WING_TURN
        * time.delta_secs();
    look.yaw_delta += turn.x;
    let was = wrap(heading.pitch - gear.state.pitch);
    let led = was + look.pitch_delta + turn.y;
    look.pitch_delta = led.clamp(was.min(-LEASH), was.max(LEASH)) - was;
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
