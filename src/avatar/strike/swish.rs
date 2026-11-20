//! Swish: the sound of a tool swung through the air, once for each swing, from where the tip is.

use bevy::prelude::*;

use super::act::{Act, Strike};
use super::held::Held;
use crate::audio::Whoosh;

/// Sends a [`Whoosh`] for every body whose swing just began, and lets the next swing sound again once
/// the swing is over.
pub fn swish(
    mut bodies: Query<(&GlobalTransform, &mut Strike, &Held)>,
    mut out: MessageWriter<Whoosh>,
) {
    for (placed, mut strike, held) in &mut bodies {
        let swinging = matches!(strike.act, Act::Swing(_));
        if swinging == strike.sounded {
            continue;
        }
        strike.sounded = swinging;
        if swinging {
            out.write(Whoosh {
                at: placed.transform_point(strike.blade[0][1]),
                speed: held.swing.speed,
                inertia: held.swing.inertia,
            });
        }
    }
}
