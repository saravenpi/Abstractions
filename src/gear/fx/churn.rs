//! What a body with gear gives the sea under it: the push of its wings and boots, its size, the
//! air around it and the jet of its boots, read by the churn of the water.

use bevy::prelude::*;

use super::super::Gear;
use super::super::jet::jetting;
use super::super::tune::{BODY_MASS, BODY_SIZE, WING_SPAN};
use crate::effects::Churn;

/// Sets the churn of every body with gear from its gear.
///
/// A body on the wings or the boots pushes its felt weight on the air; one on the ground or
/// swimming pushes nothing. Its size is the span of its wings as they spread, else its body.
pub fn feed(mut bodies: Query<(&Gear, &mut Churn)>) {
    for (gear, mut churn) in &mut bodies {
        let (state, input) = (&gear.state, &gear.input);
        let flying = f32::from(u8::from(state.mode.airborne()));
        let jet = f32::from(u8::from(jetting(input)));
        churn.push = flying * state.load * input.gravity * BODY_MASS;
        churn.size = BODY_SIZE + (WING_SPAN - BODY_SIZE) * state.spread();
        churn.air = input.density * input.air;
        churn.jet = jet * state.power();
    }
}
