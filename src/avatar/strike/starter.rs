//! Starter tools, in dev builds only until crafting exists: a key hands the local robot a spear,
//! then a club, then nothing, each built by the solver from what the planet underfoot grows.

use bevy::prelude::*;

use super::held::Held;
use super::input::Robot;
use super::stock::on;
use crate::craft::build::Intent;
use crate::world::Terrain;

/// The key that hands over the next starter tool.
pub const KEY: KeyCode = KeyCode::F8;

/// Hands the local robot the next starter tool when [`KEY`] is pressed: a spear, a club, then
/// empty hands, round again.
pub fn give(
    keys: Res<ButtonInput<KeyCode>>,
    terrain: Res<Terrain>,
    robot: Robot,
    mut turn: Local<u8>,
    mut commands: Commands,
) {
    let Some(body) = robot.body().filter(|_| keys.just_pressed(KEY)) else {
        return;
    };
    *turn = (*turn + 1) % 3;
    let intent = [None, Some(Intent::Throw), Some(Intent::Crush)][usize::from(*turn)];
    match intent.and_then(|intent| on(terrain.globe(), intent)) {
        Some(held) => {
            info!(
                "starter {:?}: {:.2} kg, {:.0} J",
                intent, held.throw.mass, held.swing.energy
            );
            commands.entity(body).insert(held);
        }
        None => {
            commands.entity(body).remove::<Held>();
        }
    }
}
