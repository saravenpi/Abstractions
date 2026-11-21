//! The beacon of the base seen from afar: the lamp on the mast of the station, drawn as a dot a
//! few pixels wide from any distance over its planet, flashing as the lamp does, so that home
//! shows from orbit by day as by night. It is the same light at the same place; near the base
//! the base is drawn with the lamp itself, and the dot hides.

use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::base::Base;
use super::glow::{Glow, flash};
use super::spawn::Sight;
use super::tune::{FAR_BEACON, SIGHT};
use super::works::Works;
use crate::screens::Screen;
use crate::space::Placed;
use crate::world::{Planet, Sited, StreamAnchor};

/// Marks the root of the far beacon, fixed to the planet at the lamp.
#[derive(Component)]
pub struct Far;

/// Marks the dot of the far beacon, sized with the distance.
#[derive(Component)]
pub struct Dot;

/// Returns the planet position of the lamp of the beacon of `base`, or nothing before the parts
/// of the base are built.
fn lamp(base: &Base, works: &Works) -> Option<DVec3> {
    let mut lamps = works.kit.iter().flat_map(|kit| &kit.lamps);
    let (_, at) = lamps.find(|lamp| lamp.0 == Glow::Beacon)?;
    Some(base.centre + base.rot() * at.as_dvec3())
}

/// Returns the bright unlit material of the dot: the colour of the lamp, [`FAR_BEACON`] times
/// white so that it blooms against a lit planet.
fn paint() -> StandardMaterial {
    let tint = (Glow::Beacon.cast().to_linear() * FAR_BEACON.2).with_alpha(1.0);
    StandardMaterial {
        base_color: Color::LinearRgba(tint),
        unlit: true,
        ..default()
    }
}

/// The assets a new dot is made of.
type Kit<'w> = (
    Option<ResMut<'w, Assets<Mesh>>>,
    Option<ResMut<'w, Assets<StandardMaterial>>>,
);

/// Spawns the far beacon while the planet of the base is streamed, and despawns it when another
/// planet is.
pub fn raise(
    (mut commands, sight): (Commands, Sight),
    (base, works, planet): (Res<Base>, Res<Works>, Option<Res<Planet>>),
    fars: Query<Entity, With<Far>>,
    (meshes, materials): Kit,
) {
    let here = planet.is_some_and(|planet| planet.body == base.body);
    if let Some(far) = fars.iter().next().filter(|_| !here) {
        commands.entity(far).despawn();
    }
    let (Some(at), Some((anchor, link))) = (lamp(&base, &works), sight.link(&base)) else {
        return;
    };
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };
    if !here || !fars.is_empty() {
        return;
    }
    let sited = Sited {
        pos: at,
        rot: base.rot(),
    };
    let placed = Placed {
        pos: link.to_anchor(sited.pos),
        rot: link.turn * sited.rot,
    };
    let transform = Transform::from_translation(anchor.to_render(placed.pos));
    let dot = (Dot, Mesh3d(meshes.add(Sphere::new(0.5))), NotShadowCaster);
    commands
        .spawn((Name::new("Far beacon"), Far, sited, placed, transform))
        .insert((Visibility::Hidden, DespawnOnEnter(Screen::Title)))
        .with_child((dot, MeshMaterial3d(materials.add(paint()))));
}

/// Sizes the dot of the far beacon to [`FAR_BEACON`] radians of the view at its distance from
/// the focus of the world, flashing with the lamp, and shows it only past the distance the base
/// is drawn within.
pub fn shine(
    (base, works, time): (Res<Base>, Res<Works>, Res<Time>),
    sight: Sight,
    mut fars: Query<&mut Visibility, With<Far>>,
    mut dots: Query<&mut Transform, (With<Dot>, Without<StreamAnchor>)>,
) {
    let distance = lamp(&base, &works).and_then(|at| sight.distance(&base, at));
    let shown = distance.is_some_and(|distance| distance > SIGHT.0);
    let wanted = [Visibility::Hidden, Visibility::Inherited][usize::from(shown)];
    for mut visibility in &mut fars {
        visibility.set_if_neq(wanted);
    }
    let pulse = FAR_BEACON.1 + (1.0 - FAR_BEACON.1) * flash(time.elapsed_secs());
    let size = (distance.unwrap_or(0.0) as f32 * FAR_BEACON.0 * pulse).max(1.0);
    for mut transform in &mut dots {
        transform.set_if_neq(Transform::from_scale(Vec3::splat(size)));
    }
}
