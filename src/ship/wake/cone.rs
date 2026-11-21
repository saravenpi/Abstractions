//! The vapour cone: a pale cone of condensed water that wraps the rear of a hull for a moment as
//! it passes the speed of sound in wet air, on the way up as on the way down.
//!
//! Every hull has its own, the ship of this machine and those of the other players alike, read
//! from the air the trails read for it: the Mach number from the temperature and the gas of the
//! air at its height, the thickness of that air, and the wetness of the clouds, the rain or the
//! nearly saturated air just under the cloud base. Cost: one cone of 64 triangles per hull,
//! drawn only while it shows.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Flown;
use crate::screens::Screen;
use crate::ship::HULL_RADIUS;
use crate::ship::exhaust::Exhaust;
use crate::space::{Anchor, Placed};
use crate::speed::cone;
use crate::speed::tune::CONE_ALPHA;

/// Share of the hull radius the cone is long, wide at its base, and sits behind the hull centre.
const SHAPE: (f32, f32, f32) = (1.7, 0.95, 0.5);

/// Linear colour of the vapour: white with a hint of blue.
const TINT: Vec3 = Vec3::new(0.93, 0.96, 1.0);

/// The cone of one hull, with the strength its material shows.
#[derive(Component, Debug, Clone, Copy)]
pub struct Vapour {
    /// The ship it belongs to.
    pub ship: Entity,
    /// Strength the material was last written with.
    shown: f32,
}

/// Gives every ship that has none its hidden cone, and takes away the cones of the ships that
/// are gone.
pub fn spawn(
    mut commands: Commands,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    ships: Query<Entity, With<Exhaust>>,
    cones: Query<(Entity, &Vapour)>,
) {
    for (entity, vapour) in &cones {
        if !ships.contains(vapour.ship) {
            commands.entity(entity).try_despawn();
        }
    }
    for ship in &ships {
        if cones.iter().any(|(_, vapour)| vapour.ship == ship) {
            continue;
        }
        let material = StandardMaterial {
            base_color: Color::linear_rgba(TINT.x, TINT.y, TINT.z, 0.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            fog_enabled: false,
            double_sided: true,
            cull_mode: None,
            ..default()
        };
        commands.spawn((
            Vapour { ship, shown: 0.0 },
            Mesh3d(meshes.add(Cone::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(material)),
            Transform::from_scale(Vec3::ZERO),
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Fits the cone around the rear of the hull at `placed`, apex toward the nose.
fn fit(transform: &mut Transform, anchor: &Anchor, placed: &Placed) {
    let nose = anchor.rotation_to_render(placed.rot) * Vec3::NEG_Z;
    let across = SHAPE.1 * HULL_RADIUS * 2.0;
    transform.translation = anchor.to_render(placed.pos) - nose * (SHAPE.2 * HULL_RADIUS);
    transform.rotation = Quat::from_rotation_arc(Vec3::Y, nose);
    transform.scale = Vec3::new(across, SHAPE.0 * HULL_RADIUS, across);
}

/// The cones, which are no hulls.
type Cones<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Vapour,
        &'static mut Transform,
        &'static MeshMaterial3d<StandardMaterial>,
    ),
    Without<Flown>,
>;

/// Fits each cone to its hull and sizes its alpha by how strongly it shows in the air the hull
/// flew in this frame, or hides it. The material is written only when the strength moves.
pub fn draw(
    anchor: Option<Res<Anchor>>,
    hulls: Query<(&Placed, &Flown)>,
    mut cones: Cones,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(anchor) = anchor else {
        return;
    };
    for (mut vapour, mut transform, handle) in &mut cones {
        let Ok((placed, flown)) = hulls.get(vapour.ship) else {
            continue;
        };
        let air = flown.air;
        let strength = cone(air.mach, air.thick, air.moist);
        if strength <= 0.0 {
            if transform.scale != Vec3::ZERO {
                transform.scale = Vec3::ZERO;
            }
            vapour.shown = 0.0;
            continue;
        }
        fit(&mut transform, &anchor, placed);
        tint(&mut vapour, strength, &handle.0, &mut materials);
    }
}

/// Writes the alpha of the cone of `vapour` for `strength`, when it moved since it was last
/// written.
fn tint(
    vapour: &mut Vapour,
    strength: f32,
    handle: &Handle<StandardMaterial>,
    materials: &mut Assets<StandardMaterial>,
) {
    if (strength - vapour.shown).abs() <= 0.002 && vapour.shown != 0.0 {
        return;
    }
    vapour.shown = strength;
    if let Some(mut material) = materials.get_mut(handle) {
        let alpha = CONE_ALPHA * strength;
        material.base_color = Color::linear_rgba(TINT.x, TINT.y, TINT.z, alpha);
    }
}
