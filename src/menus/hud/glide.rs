//! What the prompt line says about the wingsuit: the key that spreads it in a fall, and why it
//! cannot glide where the air is too thin.

use bevy::prelude::*;

use super::context::Context;
use crate::gear::{Gear, glides};
use crate::player::{Motion, Player, local};
use crate::sim::PlayerId;

/// The player bodies, their motion and their gear.
pub type Bodies<'w, 's> =
    Query<'w, 's, (&'static PlayerId, &'static Motion, &'static Gear), With<Player>>;

/// Free height under the body in metres above which a fall with the wings folded shows the key
/// that spreads them: more than a jump on flat ground rises, so a hop never shows it.
pub const OPEN_HEIGHT: f32 = 6.0;

/// The sentence the line shows while the wings are asked open in air too thin to carry them.
pub const THIN: &str = "The air is too thin to glide";

/// The words of the key that spreads the wings.
pub const SPREAD: &str = "Spread the wings";

/// Returns what the line says about the wings of the body moving as `motion` with `gear`, on a
/// planet whose air is `air` thick against the home planet, the wings spread by `jump`: the
/// block sentence while they are asked open in air too thin to glide in (see
/// [`crate::gear::glides`]), and the key that spreads them in a fall high enough and in air
/// that carries them.
pub fn said(
    motion: &Motion,
    gear: &Gear,
    air: f32,
    jump: KeyCode,
) -> (Option<String>, Option<(KeyCode, String)>) {
    let input = &gear.input;
    let carried = glides(air, input.density, input.gravity);
    let aloft = !motion.grounded && !motion.swimming;
    let block = (aloft && motion.wings && !carried).then(|| THIN.to_string());
    let high = input.clearance > OPEN_HEIGHT;
    let open = aloft && !motion.wings && high && carried;
    (block, open.then(|| (jump, SPREAD.to_string())))
}

/// Returns what the line says about the wings of the local `body`, if there is one, in
/// `context`: while gliding why the air cannot carry them, in a fall on foot the key that
/// spreads them, and nothing anywhere else, the ship included. See [`said`].
pub fn spoken(
    body: Option<(&Motion, &Gear)>,
    air: f32,
    jump: KeyCode,
    context: Context,
) -> (Option<String>, Option<(KeyCode, String)>) {
    let Some((motion, gear)) = body else {
        return (None, None);
    };
    let (block, act) = said(motion, gear, air, jump);
    match context {
        Context::Gliding => (block, None),
        Context::OnFoot => (None, act),
        _ => (None, None),
    }
}

/// Returns the motion and the gear of the body of the player at this machine among `bodies`.
pub fn mine<'a>(bodies: &'a Bodies) -> Option<(&'a Motion, &'a Gear)> {
    local(
        bodies
            .iter()
            .map(|(id, motion, gear)| (*id, (motion, gear))),
    )
}
