//! Tests that a chunk casts no shadow while it fades out, and casts again when the fade stops,
//! unless it is a shadowless node, which casts none at all.

use bevy::ecs::world::CommandQueue;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use super::dither::{Shadowless, Way};
use super::fade::Fades;
use super::horizon::{Bounds, shade};
use super::nodes::Node;

/// Starts a fade of `way` on `part`, which is its own root, and applies the commands.
fn begin(world: &mut World, fades: &mut Fades, part: Entity, way: Way) {
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);
    fades.begin(&mut commands, part, vec![part], (way, 0.0));
    queue.apply(world);
}

/// Stops the fade of `part` and applies the commands.
fn cancel(world: &mut World, fades: &mut Fades, part: Entity) {
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);
    fades.cancel(&mut commands, part);
    queue.apply(world);
}

/// A chunk that fades out or is hidden casts no shadow, and one that fades in does.
#[test]
fn a_fading_chunk_casts_no_shadow() {
    for (way, shadowless) in [(Way::Leave, true), (Way::Hide, true), (Way::In, false)] {
        let mut world = World::new();
        let part = world.spawn(Mesh3d::default()).id();
        let mut fades = Fades::default();
        begin(&mut world, &mut fades, part, way);
        let has = world.entity(part).contains::<NotShadowCaster>();
        assert_eq!(has, shadowless, "{way:?}");
    }
}

/// A fade that stops leaves the chunk casting its shadow again.
#[test]
fn a_cancelled_fade_casts_again() {
    let mut world = World::new();
    let part = world.spawn(Mesh3d::default()).id();
    let mut fades = Fades::default();
    begin(&mut world, &mut fades, part, Way::Hide);
    assert!(world.entity(part).contains::<NotShadowCaster>());
    cancel(&mut world, &mut fades, part);
    assert!(!world.entity(part).contains::<NotShadowCaster>());
}

/// A coarse node casts no shadow, and neither a fade in nor the end of a fade gives it one.
#[test]
fn a_shadowless_node_never_casts() {
    let mut world = World::new();
    let part = world
        .spawn((Mesh3d::default(), NotShadowCaster, Shadowless))
        .id();
    let mut fades = Fades::default();
    begin(&mut world, &mut fades, part, Way::In);
    assert!(world.entity(part).contains::<NotShadowCaster>());
    cancel(&mut world, &mut fades, part);
    assert!(world.entity(part).contains::<NotShadowCaster>());
    begin(&mut world, &mut fades, part, Way::Hide);
    cancel(&mut world, &mut fades, part);
    assert!(world.entity(part).contains::<NotShadowCaster>());
}

/// A coarse node spawned shadowless gets shadows back on its first near horizon pass.
#[test]
fn a_near_coarse_node_casts_and_receives_shadows() {
    let mut world = World::new();
    let entity = world
        .spawn((
            Mesh3d::default(),
            NotShadowCaster,
            NotShadowReceiver,
            Shadowless,
        ))
        .id();
    let mut node = Node {
        entity,
        visible: false,
        bounds: Bounds::WHOLE,
        beyond: false,
        shadowless: true,
    };
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    shade(&mut commands, &mut node, false, None);
    queue.apply(&mut world);
    assert!(!node.shadowless);
    assert!(!world.entity(entity).contains::<Shadowless>());
    assert!(!world.entity(entity).contains::<NotShadowCaster>());
    assert!(!world.entity(entity).contains::<NotShadowReceiver>());
}

/// Detail shadow culling strips shadow components far away and keeps `NotShadowCaster` while fading.
#[test]
fn dither_shadow_toggles_and_respects_fades() {
    use super::dither::shadow;

    let mut world = World::new();
    let entity = world.spawn(Mesh3d::default()).id();
    let mut queue = CommandQueue::default();
    shadow(&mut Commands::new(&mut queue, &world), entity, true, None);
    queue.apply(&mut world);
    assert!(world.entity(entity).contains::<Shadowless>());
    assert!(world.entity(entity).contains::<NotShadowCaster>());
    assert!(world.entity(entity).contains::<NotShadowReceiver>());
    shadow(
        &mut Commands::new(&mut queue, &world),
        entity,
        false,
        Some(Way::Hide),
    );
    queue.apply(&mut world);
    assert!(!world.entity(entity).contains::<Shadowless>());
    assert!(world.entity(entity).contains::<NotShadowCaster>());
    assert!(!world.entity(entity).contains::<NotShadowReceiver>());
    shadow(&mut Commands::new(&mut queue, &world), entity, false, None);
    queue.apply(&mut world);
    assert!(!world.entity(entity).contains::<NotShadowCaster>());
}
