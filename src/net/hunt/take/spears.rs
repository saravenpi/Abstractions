//! Spears of the others that stand in animals of this machine: drawn like the ones this robot
//! throws, so the robot can draw them out, and taken away when somebody does.

use abstractions_net::wire::Spear as Stuck;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::avatar::strike::{Held, Kit, Rest, Spear, kit, tool};
use crate::fauna::Creature;
use crate::net::hunt::{Hunted, item};
use crate::net::state::Remotes;
use crate::screens::Screen;
use crate::space::Location;
use crate::world::Terrain;

/// What drawing a spear reads and writes.
#[derive(SystemParam)]
pub struct Stage<'w, 's> {
    /// Spawns and despawns.
    commands: Commands<'w, 's>,
    /// What was sent or taken.
    hunted: ResMut<'w, Hunted>,
    /// The planet underfoot.
    terrain: Res<'w, Terrain>,
    /// Where the player is.
    location: Res<'w, Location>,
    /// The puppets of the others, whose spears these are.
    remotes: Res<'w, Remotes>,
}

/// Returns a new kit of drawn tools and keeps it as a resource.
fn made(commands: &mut Commands, stores: &mut Stores) -> Kit {
    let made = kit(&mut stores.0, &mut stores.1);
    commands.insert_resource(made.clone());
    made
}

/// The stores of meshes and materials the kit is made in.
type Stores<'a, 'b> = (
    ResMut<'a, Assets<Mesh>>,
    ResMut<'b, Assets<StandardMaterial>>,
);

/// Takes away the spears the others pulled out, and draws the ones they planted in animals that are
/// drawn here. A spear whose animal is not drawn waits; one of another body, or that cannot be
/// built on this planet, is dropped.
pub fn plant(
    mut stage: Stage,
    kit: Option<Res<Kit>>,
    mut stores: Stores,
    animals: Query<(Entity, &Creature)>,
) {
    for key in std::mem::take(&mut stage.hunted.pulled) {
        let at = stage.hunted.planted.iter().find(|(_, (k, _))| *k == key);
        if let Some(entity) = at.map(|(entity, _)| *entity) {
            stage.hunted.planted.remove(&entity);
            stage.commands.entity(entity).despawn();
        }
        stage.hunted.stuck.retain(|spear| spear.key != key);
    }
    let body = stage.location.body;
    stage.hunted.stuck.retain(|spear| spear.animal.body == body);
    let mut kit = kit.map(|kit| (*kit).clone());
    let waiting = std::mem::take(&mut stage.hunted.stuck);
    for spear in waiting {
        let found = animals
            .iter()
            .find(|(_, c)| c.herd == spear.animal.herd && c.slot == spear.animal.slot);
        let Some((creature, _)) = found else {
            stage.hunted.stuck.push(spear);
            continue;
        };
        let shown = kit.get_or_insert_with(|| made(&mut stage.commands, &mut stores));
        draw(&mut stage, shown, (&spear, creature));
    }
}

/// Draws the spear `spear` standing in the animal `creature`, as a spear that was thrown and stuck.
fn draw(stage: &mut Stage, kit: &Kit, (spear, creature): (&Stuck, Entity)) {
    let Some(item) = item::unpack(&spear.item, &stage.terrain) else {
        return;
    };
    let thrower = stage.remotes.robots.get(&spear.key.owner);
    let thrower = thrower
        .and_then(|track| track.puppet)
        .unwrap_or(Entity::PLACEHOLDER);
    let drawn = tool(
        &mut stage.commands,
        kit,
        &item.build,
        None,
        -item.build.reach(),
    );
    let flown = Spear {
        item: Held { out: true, ..item },
        velocity: Vec3::ZERO,
        rest: Rest::Stuck,
        thrower,
        body: thrower,
        was: Vec3::ZERO,
        lent: None,
    };
    let along = Vec3::from_array(spear.dir).normalize_or(Vec3::Y);
    let place = Transform::from_translation(Vec3::from_array(spear.at))
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, along));
    let marks = (ChildOf(creature), DespawnOnEnter(Screen::Title));
    stage.commands.entity(drawn).insert((flown, place, marks));
    stage.hunted.planted.insert(drawn, (spear.key, creature));
}
