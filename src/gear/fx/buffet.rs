//! The shake of the air on a body in flight: the view trembles with the dynamic pressure.

use bevy::prelude::*;

use super::FlightFx;
use crate::render::GameCamera;
use crate::speed::shake::shake;
use crate::speed::tune::BODY_SHARE;
use crate::view::ViewPrefs;

/// Run condition: the air buffets the body.
pub fn buffeted(fx: Res<FlightFx>) -> bool {
    fx.buffet > 0.0
}

/// Shakes the view the rig made by an offset and a tilt, as the ship does in rough air, scaled by
/// the buffeting of the air, by [`BODY_SHARE`] and by the "Speed effects" setting. The rig writes the
/// whole camera pose every frame, so the shake never adds up.
pub fn tremble(
    time: Res<Time>,
    fx: Res<FlightFx>,
    prefs: Res<ViewPrefs>,
    mut eyes: Single<&mut Transform, With<GameCamera>>,
) {
    let level = fx.buffet * BODY_SHARE * prefs.speed_fx.clamp(0.0, 1.0);
    if level <= 0.0 {
        return;
    }
    let (offset, tilt) = shake(time.elapsed_secs_wrapped(), level);
    let turned = eyes.rotation * offset;
    eyes.translation += turned;
    eyes.rotation = (eyes.rotation * tilt).normalize();
}
