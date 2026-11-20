//! Pulling: the interact key draws a spear out of the animal or the carcass it is stuck in, when the
//! robot stands within reach of it with empty hands.

use bevy::prelude::*;

use super::flight::{Rest, Spear};
use super::held::Held;
use super::input::Robot;
use super::tune::PULL;
use crate::craft::Pack;
use crate::fauna::Creature;
use crate::player::Player;
use crate::settings::{Persistent, Settings};

/// What the robot's reach reads: the players, the hands that are full and the animals.
pub type Reach<'w, 's> = (
    Query<'w, 's, &'static Transform, With<Player>>,
    Query<'w, 's, (), With<Held>>,
    Query<'w, 's, (), With<Creature>>,
);

/// What a pull reads and writes: the keys, the settings that bind them and the pack.
type Hand<'w> = (
    Res<'w, ButtonInput<KeyCode>>,
    Res<'w, Persistent<Settings>>,
    Option<ResMut<'w, Pack>>,
);

/// Gives the nearest spear stuck in an animal back to the local robot when the interact key goes
/// down and its hands are empty.
pub fn pull(
    mut commands: Commands,
    (keys, settings, mut pack): Hand,
    robot: Robot,
    spears: Query<(Entity, &Spear, &GlobalTransform, &ChildOf)>,
    (players, full, creatures): Reach,
) {
    if !keys.just_pressed(settings.controls.keys.ship.board) {
        return;
    }
    let Some((body, player)) = robot.pair() else {
        return;
    };
    let Ok(at) = players.get(player) else { return };
    if full.contains(body) {
        return;
    }
    let reach = |(_, spear, placed, of): &(Entity, &Spear, &GlobalTransform, &ChildOf)| {
        spear.rest == Rest::Stuck
            && creatures.contains(of.parent())
            && placed.translation().distance(at.translation) < PULL
    };
    if let Some((entity, spear, _, _)) = spears.iter().find(reach) {
        take(&mut commands, body, spear, pack.as_deref_mut());
        commands.entity(entity).despawn();
    }
}

/// Gives the item of `spear` back to `body`: into the hotbar slot it was lent from, or into the
/// hands when nothing lent it.
pub fn take(commands: &mut Commands, body: Entity, spear: &Spear, pack: Option<&mut Pack>) {
    match (spear.lent.clone(), pack) {
        (Some(lent), Some(pack)) => pack.restore(lent),
        _ => {
            commands.entity(body).insert(spear.item.clone());
        }
    }
}
