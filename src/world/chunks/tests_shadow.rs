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

/// Newly spawned detail chunks and props start shadowless until `horizon` approves shadows, and
/// LOD terrain swaps replace the mesh directly without spawning dither ghosts or `VisibilityRange`.
#[test]
fn detail_chunks_spawn_shadowless_and_remesh_without_ghosts() {
    use avian3d::prelude::Collider;
    use bevy::camera::visibility::VisibilityRange;
    use bevy::tasks::AsyncComputeTaskPool;

    use super::Chunks;
    use super::detail::{Detail, Work, refine};
    use super::plan::Loaded;
    use super::turf::Turf;
    use crate::space::Anchor;
    use crate::universe::Universe;
    use crate::world::globe::Face;
    use crate::world::ground::GroundMaterial;
    use crate::world::mesh::Lod;
    use crate::world::props::{Cover, PropAssets, spawn_props};
    use crate::world::rocks::{Stand, Taken};
    use crate::world::scatter::Rig;
    use crate::world::{Globe, Planet, SpawnPoint, Terrain, WorldReady};

    let mut app = App::new();
    let mut materials = Assets::<GroundMaterial>::default();
    let mut images = Assets::<Image>::default();
    let props = PropAssets::new(&mut materials, &mut images);
    let mat = props.material.clone();
    let globe = std::sync::Arc::new(Globe::new(7, 6000.0, 9.81));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let universe = Universe::new(7);
    let body = universe.home();
    let spawn = Vec3::ZERO;
    let anchor = Anchor::surface(body, &terrain, spawn);
    let planet = Planet {
        body,
        globe: globe.clone(),
        day_seconds: 1200.0,
    };
    app.add_plugins(MinimalPlugins)
        .insert_resource(props)
        .insert_resource(terrain.clone())
        .insert_resource(universe)
        .insert_resource(planet)
        .insert_resource(anchor)
        .insert_resource(SpawnPoint(spawn))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Chunks>()
        .init_resource::<WorldReady>()
        .add_systems(Update, refine);
    let cube = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::default());
    let chunk_id = crate::world::globe::ChunkId {
        face: Face::PosY,
        level: 4,
        x: 0,
        y: 0,
    };
    let entity = app
        .world_mut()
        .spawn((
            Mesh3d(cube.clone()),
            NotShadowCaster,
            NotShadowReceiver,
            Shadowless,
        ))
        .id();
    let cover = Cover {
        props: Some(cube),
        grass: None,
    };
    let [prop_entity, _] = spawn_props(&mut app.world_mut().commands(), entity, &mat, cover);
    let prop_entity = prop_entity.expect("props spawned");
    app.update();
    for target in [entity, prop_entity] {
        assert!(app.world().entity(target).contains::<Shadowless>());
        assert!(app.world().entity(target).contains::<NotShadowCaster>());
        assert!(app.world().entity(target).contains::<NotShadowReceiver>());
    }
    let mut detail = Detail::new(Lod::Mid, Turf::new(None, false));
    detail.begin(
        Work::Terrain(Lod::Near),
        (terrain, IVec2::ZERO),
        AsyncComputeTaskPool::get(),
    );
    let loaded = Loaded {
        entity,
        props: Some(prop_entity),
        heights: Collider::cuboid(1.0, 1.0, 1.0),
        bodies: None,
        solid: false,
        shown: true,
        detail,
        stand: Stand::of(0, &[], &Rig::PLAIN, &Taken::default()),
        center: bevy::math::DVec3::Y * 6000.0,
        shadowless: true,
    };
    app.world_mut()
        .resource_mut::<Chunks>()
        .loaded
        .insert(chunk_id, loaded);
    for _ in 0..200 {
        app.update();
        if app.world().resource::<Chunks>().loaded[&chunk_id]
            .detail
            .lod
            == Lod::Near
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let chunks = app.world().resource::<Chunks>();
    assert_eq!(chunks.loaded[&chunk_id].detail.lod, Lod::Near);
    assert!(chunks.fades.idle());
    assert!(!app.world().entity(entity).contains::<VisibilityRange>());
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<Children>()
            .map(Children::len),
        Some(1)
    );
}
