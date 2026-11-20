//! The pool of flecks: the shared mesh, the materials by colour, and the fixed set of entities.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::tune::WET;
use crate::screens::Screen;
use crate::space::Rebased;

/// Number of fleck entities, made once per world.
pub const SLOTS: usize = 192;

/// One pooled fleck.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Fleck {
    /// True while it flies.
    pub live: bool,
    /// Velocity in render space, in metres per second.
    pub velocity: Vec3,
    /// Seconds it has flown.
    pub age: f32,
    /// Radius in metres.
    pub radius: f32,
    /// What the thrower tagged it with.
    pub tag: Option<u16>,
}

/// The shared mesh, the materials made so far and how many flecks fly.
#[derive(Resource, Debug, Default)]
pub struct Flecks {
    /// Unit sphere, a metre across.
    mesh: Handle<Mesh>,
    /// The material of each colour, in 8 bits a channel, wet or not.
    looks: Vec<([u8; 3], bool, Handle<StandardMaterial>)>,
    /// Flecks that fly now.
    pub live: usize,
}

impl Flecks {
    /// Returns the material of `colour` (linear RGB), made on first use: one for each colour in 8
    /// bits, so a planet whose animals bleed alike makes a few.
    pub fn look(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        colour: [f32; 3],
        wet: bool,
    ) -> Handle<StandardMaterial> {
        let key = colour.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
        if let Some((_, _, handle)) = self.looks.iter().find(|(k, w, _)| *k == key && *w == wet) {
            return handle.clone();
        }
        let handle = materials.add(StandardMaterial {
            base_color: Color::linear_rgb(colour[0], colour[1], colour[2]),
            perceptual_roughness: if wet { WET } else { 1.0 },
            ..default()
        });
        self.looks.push((key, wet, handle.clone()));
        handle
    }

    /// Returns the shared mesh.
    pub fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }
}

/// Run condition: some fleck flies.
pub fn flying(flecks: Res<Flecks>) -> bool {
    flecks.live > 0
}

/// Returns the slot a new fleck takes: a free one, else the one that has flown longest. `ages` holds
/// each slot's age, or nothing for a free one.
pub fn claim(ages: impl Iterator<Item = Option<f32>>) -> usize {
    let mut best = (0, f32::NEG_INFINITY);
    for (slot, age) in ages.enumerate() {
        let score = age.unwrap_or(f32::INFINITY);
        if score > best.1 {
            best = (slot, score);
        }
    }
    best.0
}

/// Makes the shared mesh and spawns the flecks when gameplay starts, unless they exist. They are free
/// entities in render space, marked to follow anchor rebases.
pub fn spawn(
    mut commands: Commands,
    mut flecks: ResMut<Flecks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<(), With<Fleck>>,
) {
    if !existing.is_empty() {
        return;
    }
    flecks.mesh = meshes.add(Sphere::new(0.5).mesh().uv(8, 6));
    flecks.looks.clear();
    flecks.live = 0;
    let look = flecks.look(&mut materials, [0.5; 3], false);
    for _ in 0..SLOTS {
        commands.spawn((
            Fleck::default(),
            Mesh3d(flecks.mesh.clone()),
            MeshMaterial3d(look.clone()),
            Transform::from_scale(Vec3::ZERO),
            Visibility::Hidden,
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}
