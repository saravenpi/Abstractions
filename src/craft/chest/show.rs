//! Spawning, rebasing, and despawning placed chests and flickering torches on the active planet.

use abstractions_net::chest::{PlaceableKind, PlacedProp};
use avian3d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::math::{DQuat, DVec3};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::ChestWorld;
use crate::craft::gun::mesh::{Tri, push_box, push_prism};
use crate::player::Player;
use crate::screens::Screen;
use crate::space::{Anchor, Frame, Placed};
use crate::world::Terrain;

/// Maximum distance in metres at which placed chests and torches are spawned.
const DRAW_RANGE: f64 = 96.0;

/// Base luminous intensity of a standing torch flame in lumens.
const TORCH_LUMENS: f32 = 3800.0;

/// Marker on the root entity of a spawned placed chest or torch.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropEntity {
    /// World prop identifier from [`PlacedProp::id`].
    pub id: u64,
    /// Whether this entity is a chest or a torch.
    pub kind: PlaceableKind,
}

/// Marker on a torch's warm flickering point light.
#[derive(Component, Debug, Clone, Copy)]
pub struct TorchFlame {
    /// Deterministic phase offset so neighboring torches do not flicker in unison.
    pub phase: f32,
}

/// Cached procedural meshes and vertex-colored material shared by all placed chests and torches.
#[derive(Resource, Clone)]
pub(super) struct PropAssets {
    /// Wooden plank box mesh with iron bands and front latch (`0.72 x 0.48 x 0.50` m).
    chest: Handle<Mesh>,
    /// Wooden pole mesh with iron collar and ember head (`0.85` m tall).
    torch: Handle<Mesh>,
    /// Vertex-colored matte material.
    mat: Handle<StandardMaterial>,
}

/// Converts flat-shaded triangles into a vertex-colored Bevy [`Mesh`].
fn tris_to_mesh(tris: &[Tri]) -> Mesh {
    let n = tris.len() * 3;
    let (mut pos, mut norm, mut col) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for tri in tris {
        let normal = (tri.v[1] - tri.v[0])
            .cross(tri.v[2] - tri.v[0])
            .normalize_or(Vec3::Y);
        let rgba = [tri.color[0], tri.color[1], tri.color[2], 1.0];
        for v in tri.v {
            pos.push(v.to_array());
            norm.push(normal.to_array());
            col.push(rgba);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, norm)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
    .with_inserted_indices(Indices::U32((0..n as u32).collect()))
}

/// Builds the wooden chest mesh (`0.72 x 0.48 x 0.50` m) with dark iron bands and brass latch.
fn chest_mesh() -> Mesh {
    let mut tris = Vec::with_capacity(64);
    let boxes = [
        (
            Vec3::new(-0.36, -0.24, -0.25),
            Vec3::new(0.36, 0.24, 0.25),
            [0.46, 0.27, 0.12],
        ),
        (
            Vec3::new(-0.365, 0.08, -0.255),
            Vec3::new(0.365, 0.11, 0.255),
            [0.18, 0.19, 0.22],
        ),
        (
            Vec3::new(-0.28, -0.245, -0.258),
            Vec3::new(-0.22, 0.245, 0.258),
            [0.18, 0.19, 0.22],
        ),
        (
            Vec3::new(0.22, -0.245, -0.258),
            Vec3::new(0.28, 0.245, 0.258),
            [0.18, 0.19, 0.22],
        ),
        (
            Vec3::new(-0.045, 0.01, 0.245),
            Vec3::new(0.045, 0.13, 0.275),
            [0.76, 0.58, 0.18],
        ),
    ];
    for (lo, hi, color) in boxes {
        push_box(&mut tris, lo, hi, color);
    }
    tris_to_mesh(&tris)
}

/// Builds the standing torch mesh (`0.85` m tall) with wooden shaft, iron collar, and ember head.
fn torch_mesh() -> Mesh {
    let mut tris = Vec::with_capacity(64);
    let segs = [
        (
            Vec3::new(0.0, -0.425, 0.0),
            Vec3::new(0.0, 0.26, 0.0),
            (0.04, 0.05),
            8,
            [0.38, 0.22, 0.10],
        ),
        (
            Vec3::new(0.0, 0.20, 0.0),
            Vec3::new(0.0, 0.27, 0.0),
            (0.058, 0.062),
            8,
            [0.18, 0.19, 0.22],
        ),
        (
            Vec3::new(0.0, 0.26, 0.0),
            Vec3::new(0.0, 0.425, 0.0),
            (0.072, 0.015),
            6,
            [0.98, 0.52, 0.08],
        ),
    ];
    for (a, b, r, sides, color) in segs {
        push_prism(&mut tris, a, b, r, sides, color);
    }
    tris_to_mesh(&tris)
}

/// Computes the anchor-frame `(pos, rot)` of `prop` resting on `terrain`.
fn prop_pose(prop: &PlacedProp, anchor: &Anchor, terrain: &Terrain) -> (DVec3, DQuat) {
    let dir = DVec3::from(prop.dir.map(f64::from)).normalize_or(DVec3::Y);
    let base = if anchor.flat {
        DVec3::new(
            dir.x,
            f64::from(terrain.height(dir.x as f32, dir.z as f32)),
            dir.z,
        )
    } else {
        dir * (terrain.globe().radius() + f64::from(terrain.globe().height(dir)))
    };
    let up = if anchor.flat { DVec3::Y } else { dir };
    let half_h = match prop.kind {
        PlaceableKind::Chest => 0.24,
        PlaceableKind::Torch => 0.425,
    };
    let rot = DQuat::from_rotation_arc(DVec3::Y, up) * DQuat::from_rotation_y(f64::from(prop.yaw));
    (base + up * half_h, rot)
}

/// Spawns one placed chest or torch entity with mesh, collider, and torch point light.
fn spawn_prop(
    commands: &mut Commands,
    assets: &PropAssets,
    prop: &PlacedProp,
    anchor: &Anchor,
    terrain: &Terrain,
) {
    let (pos, rot) = prop_pose(prop, anchor, terrain);
    let shown = Transform::from_translation(anchor.to_render(pos))
        .with_rotation(anchor.rotation_to_render(rot));
    let (mesh, collider, name) = match prop.kind {
        PlaceableKind::Chest => (
            assets.chest.clone(),
            Collider::cuboid(0.72, 0.48, 0.50),
            "Placed chest",
        ),
        PlaceableKind::Torch => (
            assets.torch.clone(),
            Collider::cuboid(0.14, 0.85, 0.14),
            "Placed torch",
        ),
    };
    let root = commands
        .spawn((
            PropEntity {
                id: prop.id,
                kind: prop.kind,
            },
            Name::new(name),
            DespawnOnExit(Screen::Gameplay),
            Placed { pos, rot },
            shown,
            Visibility::default(),
            RigidBody::Static,
            collider,
            Mesh3d(mesh),
            MeshMaterial3d(assets.mat.clone()),
        ))
        .id();
    if prop.kind == PlaceableKind::Torch {
        let light = PointLight {
            color: Color::srgb(1.0, 0.62, 0.24),
            intensity: TORCH_LUMENS,
            range: 18.0,
            shadow_maps_enabled: false,
            ..default()
        };
        commands.spawn((
            TorchFlame {
                phase: (prop.id as f32) * 1.618,
            },
            light,
            Transform::from_xyz(0.0, 0.48, 0.0),
            ChildOf(root),
        ));
    }
}

/// Builds a fresh [`PropAssets`] cache in `meshes` and `materials`.
fn make_assets(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> PropAssets {
    PropAssets {
        chest: meshes.add(chest_mesh()),
        torch: meshes.add(torch_mesh()),
        mat: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.78,
            ..default()
        }),
    }
}

/// Asset stores needed to build or reuse [`PropAssets`].
type Stores<'w> = (
    Option<Res<'w, PropAssets>>,
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
);

/// Player transform and drawn prop entities for proximity streaming.
type DrawnProps<'w, 's> = (
    Query<'w, 's, &'static Transform, With<Player>>,
    Query<'w, 's, (Entity, &'static PropEntity, &'static Placed)>,
);

/// Spawns nearby placed props on the active planet and despawns removed or distant ones.
pub(super) fn sync(
    mut commands: Commands,
    (world, anchor, terrain): (Res<ChestWorld>, Res<Anchor>, Option<Res<Terrain>>),
    (cached, mut meshes, mut materials): Stores,
    (robot, drawn): DrawnProps,
    mut seen: Local<Vec<u64>>,
) {
    let here = match (anchor.frame, robot.single(), terrain.as_deref()) {
        (Frame::Surface(body), Ok(at), Some(t)) => Some((body, anchor.to_frame(at.translation), t)),
        _ => None,
    };
    seen.clear();
    for (entity, prop, placed) in &drawn {
        let keep = here.is_some_and(|(body, me, _)| {
            world
                .chests
                .find(prop.id)
                .is_some_and(|p| p.body == body && placed.pos.distance(me) < DRAW_RANGE * 1.25)
        });
        if keep {
            seen.push(prop.id);
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some((body, me, terrain)) = here else {
        return;
    };
    let mut assets = cached.map(|a| (*a).clone());
    let had = assets.is_some();
    let fresh = world
        .chests
        .props
        .iter()
        .filter(|p| p.body == body && !seen.contains(&p.id));
    for prop in fresh {
        if prop_pose(prop, &anchor, terrain).0.distance(me) <= DRAW_RANGE {
            let kit = assets.get_or_insert_with(|| make_assets(&mut meshes, &mut materials));
            spawn_prop(&mut commands, kit, prop, &anchor, terrain);
        }
    }
    if let Some(kit) = assets.filter(|_| !had) {
        commands.insert_resource(kit);
    }
}

/// Smoothly flickers every active torch [`PointLight`] with a warm firelight curve.
pub(super) fn flicker(time: Res<Time>, mut flames: Query<(&TorchFlame, &mut PointLight)>) {
    let t = time.elapsed_secs();
    for (flame, mut light) in &mut flames {
        let p = t * 5.2 + flame.phase;
        let wave = 0.09 * p.sin() + 0.05 * (p * 2.31).sin() + 0.03 * (p * 4.7).cos();
        light.intensity = TORCH_LUMENS * (1.0 + wave);
    }
}
