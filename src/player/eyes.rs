//! The eyes of the body: what the camera rig follows on foot and in flight.

use bevy::prelude::*;

use super::controller::Motion;
use super::upright::tilt;
use super::{Player, PlayerSet};
use crate::gear::{FlightState, Gear};
use crate::screens::playing;
use crate::space::{Anchor, on_foot, up};
use crate::view::{Kind, Target, ViewSet};

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

/// Registers the feed of the camera rig: after the controller moved the body, before the rig.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Target>().add_systems(
        Update,
        aim.after(PlayerSet::Move)
            .before(ViewSet::Rig)
            .run_if(playing)
            .run_if(on_foot),
    );
}
