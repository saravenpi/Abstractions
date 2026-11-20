//! The loose panel: a plate that has come off its hinge on a badly hurt part and swings from it.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::super::animate::Skeleton;
use super::super::own::Own;
use super::plates;
use super::scar::Scars;
use super::tune::{LOOSE, PANEL, PANEL_RATE, PANEL_SWING};
use crate::condition::{Condition, Part};

/// The hinge of a loose panel of `part`, on the segment that carries it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Loose(pub Part);

/// Returns the way out from the plate of `part`: the back for a wing, else the side of the body
/// it is on.
fn outward(part: Part) -> Vec3 {
    match part {
        Part::WingL | Part::WingR => Vec3::Z,
        _ if part as usize >= 2 && (part as usize).is_multiple_of(2) => Vec3::NEG_X,
        _ => Vec3::X,
    }
}

/// Hangs a panel on each part hurt below [`LOOSE`] and takes it off a part that was mended.
pub fn hang(
    mut commands: Commands,
    (condition, scars): (Res<Condition>, Res<Scars>),
    (own, bodies): (Own, Query<&Skeleton>),
    hinges: Query<(Entity, &Loose)>,
) {
    let Some(skeleton) = own.body().and_then(|(_, body)| bodies.get(body).ok()) else {
        return;
    };
    for part in Part::ALL {
        let hurt = condition.of(part) < LOOSE;
        let hung = hinges.iter().find(|(_, hinge)| hinge.0 == part);
        match (hurt, hung) {
            (false, Some((entity, _))) => commands.entity(entity).despawn(),
            (true, None) => open(&mut commands, &scars, skeleton, part),
            _ => {}
        }
    }
}

/// Puts the hinge of a panel at the top edge of the first plate of `part`, with the panel hanging
/// from it.
fn open(commands: &mut Commands, scars: &Scars, skeleton: &Skeleton, part: Part) {
    let Some(plate) = plates::of(part).next() else {
        return;
    };
    let out = outward(part);
    let at = plate.centre + out * plate.half.abs().dot(out.abs()) + Vec3::Y * plate.half.y * 0.6;
    let hinge = commands
        .spawn((
            Loose(part),
            Transform::from_translation(at),
            Visibility::default(),
            ChildOf(skeleton.0[plate.seg as usize]),
        ))
        .id();
    let panel = Transform::from_xyz(0.0, -PANEL[1] * 0.5, 0.0).with_scale(Vec3::from_array(PANEL));
    commands.spawn((
        Mesh3d(scars.cube.clone()),
        MeshMaterial3d(scars.panel.clone()),
        panel,
        NotShadowCaster,
        ChildOf(hinge),
    ));
}

/// Swings every loose panel on its hinge, wider the worse its part is hurt.
pub fn sway(
    time: Res<Time>,
    condition: Res<Condition>,
    mut hinges: Query<(&Loose, &mut Transform)>,
) {
    for (Loose(part), mut at) in &mut hinges {
        let wave = (time.elapsed_secs() * PANEL_RATE + *part as u32 as f32 * 1.7).sin();
        let worse = 1.0 - condition.of(*part);
        let open = PANEL_SWING[0] + PANEL_SWING[1] * wave * worse;
        at.rotation = Quat::from_axis_angle(Vec3::Y.cross(outward(*part)), open);
    }
}
