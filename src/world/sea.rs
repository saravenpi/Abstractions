//! The sea: one wavy, foamy water mesh that follows the focus on a snapped grid.

mod depth;
mod material;
mod mesh;
mod uniform;
mod waves;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::pbr::DistanceFog;
use bevy::prelude::*;

use self::material::WaterMaterial;
use self::uniform::{FogLook, WaterUniform};
use super::SEA_LEVEL;
use super::StreamAnchor;
use super::focus::Focus;
use crate::render::{GameCamera, Palette};
use crate::screens::Screen;
use crate::wind::Wind;

pub(super) use self::material::plugin;

/// Marks the sea mesh.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sea;

/// Everything the water shader values are computed from.
#[derive(SystemParam)]
pub struct SeaInputs<'w, 's> {
    palette: Res<'w, Palette>,
    wind: Res<'w, Wind>,
    time: Res<'w, Time>,
    fog: Query<'w, 's, &'static DistanceFog, With<GameCamera>>,
}

impl SeaInputs<'_, '_> {
    /// Returns the shader values for the current sky, wind, time and camera fog.
    fn uniform(&self) -> WaterUniform {
        let fog = FogLook::from_fog(self.fog.iter().next(), self.palette.horizon);
        WaterUniform::new(&self.palette, &self.wind, self.time.elapsed_secs(), &fog)
    }
}

/// Spawns the sea when the world starts loading.
pub fn spawn_sea(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    inputs: SeaInputs,
) {
    let water = inputs.uniform();
    commands.spawn((
        Sea,
        Mesh3d(meshes.add(mesh::sea_mesh())),
        MeshMaterial3d(materials.add(WaterMaterial { water })),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0),
        NotShadowCaster,
        NotShadowReceiver,
        NoFrustumCulling,
        DespawnOnEnter(Screen::Title),
    ));
}

/// What the follow system reads from the sea entity.
type SeaItem = (
    &'static mut Transform,
    &'static MeshMaterial3d<WaterMaterial>,
);

/// Keeps the sea query disjoint from the streaming anchor query.
type SeaFilter = (With<Sea>, Without<StreamAnchor>);

/// Moves the sea under the focus on the fine grid and refreshes the shader values.
pub fn follow_focus(
    focus: Focus,
    inputs: SeaInputs,
    mut sea: Query<SeaItem, SeaFilter>,
    mut materials: ResMut<Assets<WaterMaterial>>,
) {
    let water = inputs.uniform();
    for (mut transform, handle) in &mut sea {
        if let Some(p) = focus.position() {
            let snapped = mesh::snap(p);
            transform.translation = Vec3::new(snapped.x, SEA_LEVEL, snapped.y);
        }
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.water = water;
        }
    }
}
