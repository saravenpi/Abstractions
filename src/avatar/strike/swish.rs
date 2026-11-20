//! Swish: the sound of a tool swung through the air, once for each swing, from where the tip is, and
//! the swing that met nothing, which animals nearby see and hear.

use bevy::prelude::*;

use super::act::{Act, Strike};
use super::held::Held;
use crate::audio::Whoosh;

/// A swing that ended having met nothing: where its tip was, and the player that swung it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Missed {
    /// Where the tip was at the end, in render space.
    pub at: Vec3,
    /// The player entity that swung.
    pub by: Entity,
}

/// Sends a [`Whoosh`] for every body whose swing just began, and lets the next swing sound again once
/// the swing is over, sending a [`Missed`] when it landed nothing.
pub fn swish(
    mut bodies: Query<(&GlobalTransform, &mut Strike, &Held, &ChildOf)>,
    (mut out, mut missed): (MessageWriter<Whoosh>, MessageWriter<Missed>),
) {
    for (placed, mut strike, held, of) in &mut bodies {
        let swinging = matches!(strike.act, Act::Swing(_));
        if swinging == strike.sounded {
            continue;
        }
        strike.sounded = swinging;
        let at = placed.transform_point(strike.blade[0][1]);
        if swinging {
            out.write(Whoosh {
                at,
                speed: held.swing.speed,
                inertia: held.swing.inertia,
            });
        } else if !strike.landed {
            missed.write(Missed {
                at,
                by: of.parent(),
            });
        }
    }
}
