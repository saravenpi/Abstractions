//! Water on the canopy: in cloud and in rain small drops gather on the glass of the ship this
//! machine flies, bead and creep down it at rest, stream back over it drawn out into streaks at
//! speed, and clear a few seconds after the ship leaves the wet air. They lie on the panes of
//! the glass, a few millimetres out, so they move with the canopy and the hull and never with
//! the camera, and they draw under the interface like the rest of the world.
//!
//! Cost: one mesh of 96 drops of 4 triangles, rewritten per frame only while the glass is wet.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::canopy::Canopy;
use super::wet::{DROPS, Seed, placed, seeds, stretch, wetted};
use crate::render::{CloudCrossing, Palette};
use crate::ship::Ship;
use crate::ship::telemetry::ShipTelemetry;
use crate::weather::Weather;

/// Vertices of one drop: its middle, then the four tips of its diamond.
const STRIDE: usize = 5;

/// Alpha of the middle of a drop.
const ALPHA: f32 = 0.45;

/// The drops of one canopy, with how wet the glass is and their vertex buffers.
#[derive(Component, Debug, Clone)]
pub struct Drops {
    /// Wetness of the glass, from 0 to 1.
    pub wet: f32,
    /// Where each drop starts.
    seeds: Vec<Seed>,
    /// Positions of the vertices, in the axes of the hinge.
    positions: Vec<[f32; 3]>,
    /// Linear colours of the vertices with alpha.
    colours: Vec<[f32; 4]>,
}

impl Default for Drops {
    /// Dry glass, its drops seeded and its buffers made once.
    fn default() -> Self {
        Self {
            wet: 0.0,
            seeds: seeds(),
            positions: vec![[0.0; 3]; DROPS * STRIDE],
            colours: vec![[0.0; 4]; DROPS * STRIDE],
        }
    }
}

impl Drops {
    /// Lays every drop at `seconds` and the airspeed `speed` in the linear colour `tint`: the
    /// drops the wetness reaches show, the others are clear.
    fn lay(&mut self, seconds: f32, speed: f32, tint: Vec3) {
        let long = stretch(speed);
        for (index, seed) in self.seeds.iter().enumerate() {
            let (middle, along) = placed(seed, seconds, speed);
            let side = seed.normal.cross(along);
            let alpha = ALPHA * f32::from(u8::from(seed.order < self.wet));
            let tips =
                [along * long, side, -along * long, -side].map(|tip| middle + tip * seed.size);
            let first = index * STRIDE;
            self.positions[first] = middle.to_array();
            self.colours[first] = tint.extend(alpha).to_array();
            for (corner, tip) in tips.iter().enumerate() {
                self.positions[first + 1 + corner] = tip.to_array();
                self.colours[first + 1 + corner] = tint.extend(0.0).to_array();
            }
        }
    }

    /// Copies the buffers into `mesh` without changing their size.
    fn write(&self, mesh: &mut Mesh) {
        use bevy::mesh::VertexAttributeValues::{Float32x3, Float32x4};
        if let Some(Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            positions.copy_from_slice(&self.positions);
        }
        if let Some(Float32x4(colours)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            colours.copy_from_slice(&self.colours);
        }
    }
}

/// Returns the empty mesh of the drops.
fn mesh() -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let count = DROPS * STRIDE;
    let faces = (0..DROPS as u32).flat_map(|drop| {
        let first = drop * STRIDE as u32;
        (0..4u32).flat_map(move |tip| [first, first + 1 + tip, first + 1 + (tip + 1) % 4])
    });
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; count])
    .with_inserted_indices(Indices::U32(faces.collect()))
}

/// Marks a canopy hinge that has its drops.
#[derive(Component, Debug, Clone, Copy)]
pub struct Wetted;

/// Gives the canopy of the ship this machine flies its drops, once.
pub fn spawn(
    mut commands: Commands,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    hinges: Query<(Entity, &ChildOf), (With<Canopy>, Without<Wetted>)>,
    ships: Query<(), With<Ship>>,
) {
    for (hinge, parent) in &hinges {
        if !ships.contains(parent.parent()) {
            continue;
        }
        let material = StandardMaterial {
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            fog_enabled: false,
            double_sided: true,
            cull_mode: None,
            ..default()
        };
        let look = (
            Mesh3d(meshes.add(mesh())),
            MeshMaterial3d(materials.add(material)),
        );
        let drops = (
            Drops::default(),
            look,
            Transform::IDENTITY,
            Visibility::Hidden,
            NotShadowCaster,
        );
        commands.entity(hinge).insert(Wetted).with_child(drops);
    }
}

/// What wets the glass: the cloud and the rain at the camera, the airspeed and the light.
type Air<'w> = (
    Option<Res<'w, CloudCrossing>>,
    Option<Res<'w, Weather>>,
    Res<'w, ShipTelemetry>,
    Option<Res<'w, Palette>>,
);

/// Returns the linear colour of the sRGB colour `srgb`.
fn linear(srgb: Vec3) -> Vec3 {
    let light = Color::srgb(srgb.x, srgb.y, srgb.z).to_linear();
    Vec3::new(light.red, light.green, light.blue)
}

/// Wets or dries the glass and lays its drops, or hides them once it is dry.
pub fn drip(
    time: Res<Time>,
    (cloud, weather, telemetry, palette): Air,
    mut drops: Query<(&mut Drops, &Mesh3d, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let depth = cloud.map_or(0.0, |cloud| cloud.depth);
    let soak = depth.max(weather.map_or(0.0, |weather| weather.rain));
    let speed = telemetry.surface_speed as f32;
    let tint = palette.map_or(Vec3::splat(0.8), |palette| linear(palette.cloud));
    for (mut drops, handle, mut visibility) in &mut drops {
        drops.wet = wetted(drops.wet, soak, speed, time.delta_secs());
        if drops.wet <= 0.0 {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        drops.lay(time.elapsed_secs_wrapped(), speed, tint);
        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            drops.write(&mut mesh);
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}
