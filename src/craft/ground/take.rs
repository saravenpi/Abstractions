//! Taking: the interact key takes the thing on the ground the robot looks at, within its reach,
//! into the pack, as much as the pack has room for. The prompt names the thing and its mass.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::show::Lies;
use super::tune::{AIM, REACH};
use super::{Ground, Note, Shed};
use crate::craft::{Burden, Pack};
use crate::player::{Interact, Player, Verb};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};

/// What a take reads: the keys and their bindings, the action the interact key does, the eyes,
/// the robot and the things drawn on the ground.
#[derive(SystemParam)]
pub struct Hands<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    settings: Res<'w, Persistent<Settings>>,
    interact: ResMut<'w, Interact>,
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    lying: Query<'w, 's, (&'static Lies, &'static GlobalTransform)>,
}

impl Hands<'_, '_> {
    /// Returns the thing the robot looks at within [`REACH`]: of the things ahead of the eyes, the
    /// nearest to the line of sight, no farther than [`AIM`] from it.
    fn aimed(&self) -> Option<Lies> {
        let eye = self.camera.single().ok()?;
        let me = self.robot.single().ok()?.translation;
        let (from, dir) = (eye.translation(), eye.forward().as_vec3());
        let off = |at: Vec3| {
            let along = (at - from).dot(dir);
            (along > 0.0).then(|| (at - from - dir * along).length())
        };
        self.lying
            .iter()
            .filter(|(_, at)| at.translation().distance(me) <= REACH)
            .filter_map(|(lies, at)| Some((off(at.translation())?, *lies)))
            .filter(|(miss, _)| *miss <= AIM)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, lies)| lies)
    }
}

/// Offers to take the thing the robot looks at, and takes it when the interact key goes down while
/// the prompt names the take. A pack without room for it says so in the banner.
pub(super) fn take(
    mut hands: Hands,
    (mut ground, mut pack, mut shed): (ResMut<Ground>, ResMut<Pack>, ResMut<Shed>),
    burden: Res<Burden>,
    mut notes: MessageWriter<Note>,
) {
    let Some(lies) = hands.aimed() else {
        return;
    };
    let grams = (lies.mass * 1e3).round() as u32;
    hands.interact.weighed(Verb::Take, lies.name, Some(grams));
    let key = hands.settings.controls.keys.ship.board;
    let named = hands
        .interact
        .chosen()
        .is_some_and(|o| o.thing == lies.name);
    if !named || !hands.interact.pressed(Verb::Take, &hands.keys, key) {
        return;
    }
    let into = (&mut *pack, &mut *shed);
    if let Err(line) = ground.reach(lies.id, burden.room(), into) {
        notes.write(Note(line));
    }
}
