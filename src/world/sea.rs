//! The sea: one large translucent plane at sea level that follows the focus on a coarse grid.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::focus::Focus;
use super::{SEA_LEVEL, StreamAnchor};
use crate::screens::Screen;

/// Side length of the sea plane.
const SEA_SIZE: f32 = 4096.0;

/// Grid the plane snaps to while following the focus.
const SNAP: f32 = 64.0;

/// Marks the sea plane.
#[derive(Component, Debug, Clone, Copy)]
pub struct Sea;

/// Spawns the sea plane when the world starts loading.
pub fn spawn_sea(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let material = mats.add(StandardMaterial {
        base_color: Color::srgba(0.10, 0.38, 0.55, 0.72),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.12,
        reflectance: 0.6,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    commands.spawn((
        Sea,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(SEA_SIZE, SEA_SIZE))),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, SEA_LEVEL, 0.0),
        NotShadowCaster,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Moves the sea plane under the focus, snapped so the motion is never visible.
pub fn follow_focus(
    focus: Focus,
    mut sea: Query<&mut Transform, (With<Sea>, Without<StreamAnchor>)>,
) {
    let (Some(p), Ok(mut t)) = (focus.position(), sea.single_mut()) else {
        return;
    };
    t.translation = Vec3::new(
        (p.x / SNAP).round() * SNAP,
        SEA_LEVEL,
        (p.z / SNAP).round() * SNAP,
    );
}
