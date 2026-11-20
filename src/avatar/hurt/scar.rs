//! Scars: the dents and the scorch that pile up on the plates of a hurt part. Each is a thin slab
//! lying on a face of a plate, placed from a roll that depends on the part and its number, so the
//! same harm looks the same on every machine and adding harm only adds marks.

use std::collections::HashSet;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::super::animate::Skeleton;
use super::super::looks::paint;
use super::super::own::Own;
use super::plates::{self, Plate};
use super::tune::{MARK, MARK_THIN, SCARS};
use crate::condition::{Condition, Part, luck, tune::DAMAGED};

/// The meshes and materials of the marks of harm, which every body shares.
#[derive(Resource, Debug, Clone)]
pub struct Scars {
    /// A unit cube, scaled into slabs, panels and sparks.
    pub cube: Handle<Mesh>,
    /// The grey of a dent.
    pub dent: Handle<StandardMaterial>,
    /// The black of scorch.
    pub scorch: Handle<StandardMaterial>,
    /// The off-white of a loose panel.
    pub panel: Handle<StandardMaterial>,
    /// The hot white of a spark, bright enough to bloom.
    pub spark: Handle<StandardMaterial>,
}

/// One mark of harm, a child of the segment it lies on.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scar {
    /// The part it shows the harm of.
    pub part: Part,
    /// Its number among the marks of that part.
    pub slot: u8,
}

/// Returns the number of marks a part of health `health` carries.
pub fn count(health: f32) -> usize {
    match health < DAMAGED {
        true => ((1.0 - health.max(0.0)) * SCARS as f32).ceil() as usize,
        false => 0,
    }
}

/// Returns the slab of the mark `slot` of `part` on `plate`: on a face chosen by the roll, turned to
/// lie flat on it, a hair above it.
pub fn slab(plate: &Plate, part: Part, slot: usize) -> Transform {
    let key = (part as u32) << 8 | slot as u32;
    let axis = (luck(key) * 3.0) as usize % 3;
    let sign = if luck(key ^ 0x55) < 0.5 { -1.0 } else { 1.0 };
    let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut at = plate.centre;
    at[axis] += sign * (plate.half[axis] + MARK_THIN * 0.5);
    at[a] += (luck(key ^ 0xa1) * 1.6 - 0.8) * plate.half[a];
    at[b] += (luck(key ^ 0xb2) * 1.6 - 0.8) * plate.half[b];
    let mut normal = Vec3::ZERO;
    normal[axis] = sign;
    let edge = MARK[0] + (MARK[1] - MARK[0]) * luck(key ^ 0xc3);
    Transform::from_translation(at)
        .with_rotation(Quat::from_rotation_arc(Vec3::Z, normal))
        .with_scale(Vec3::new(edge, edge * 0.7, MARK_THIN))
}

/// Returns the mark `slot` of `part`.
fn scar_of(part: Part, slot: usize) -> Scar {
    Scar {
        part,
        slot: slot as u8,
    }
}

/// Returns every mark the harm of `condition` calls for.
fn wanted(condition: &Condition) -> impl Iterator<Item = Scar> + '_ {
    Part::ALL
        .into_iter()
        .flat_map(move |part| (0..count(condition.of(part))).map(move |slot| scar_of(part, slot)))
}

/// Draws `scar` on one plate of its part, on the segment of the body `skeleton` that carries it.
fn draw(commands: &mut Commands, scars: &Scars, skeleton: &Skeleton, scar: Scar) {
    let plates: Vec<&Plate> = plates::of(scar.part).collect();
    let key = (scar.part as u32) << 4 | u32::from(scar.slot);
    let plate = plates[(luck(key) * plates.len() as f32) as usize % plates.len()];
    let paint = match scar.slot % 2 {
        0 => &scars.scorch,
        _ => &scars.dent,
    };
    commands.spawn((
        scar,
        Mesh3d(scars.cube.clone()),
        MeshMaterial3d(paint.clone()),
        slab(plate, scar.part, usize::from(scar.slot)),
        Visibility::default(),
        NotShadowCaster,
        ChildOf(skeleton.0[plate.seg as usize]),
    ));
}

/// Draws and removes the marks of the body of the robot to match its condition: each part carries
/// as many marks as its harm calls for, scorch and dents in turn.
pub fn mark(
    mut commands: Commands,
    (condition, scars): (Res<Condition>, Res<Scars>),
    (own, bodies): (Own, Query<&Skeleton>),
    marks: Query<(Entity, &Scar)>,
) {
    let Some(skeleton) = own.body().and_then(|(_, body)| bodies.get(body).ok()) else {
        return;
    };
    let have: HashSet<Scar> = marks.iter().map(|(_, scar)| *scar).collect();
    for (entity, scar) in &marks {
        if usize::from(scar.slot) >= count(condition.of(scar.part)) {
            commands.entity(entity).despawn();
        }
    }
    for scar in wanted(&condition).filter(|scar| !have.contains(scar)) {
        draw(&mut commands, &scars, skeleton, scar);
    }
}

impl Scars {
    /// Creates the marks' cube and materials.
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let lit = |grey: f32| paint(Color::srgb(grey, grey, grey), true, None);
        Self {
            cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            dent: materials.add(lit(0.45)),
            scorch: materials.add(lit(0.03)),
            panel: materials.add(lit(0.8)),
            spark: materials.add(paint(Color::linear_rgb(9.0, 6.0, 2.5), false, None)),
        }
    }
}

/// Makes the marks' meshes and materials once the asset stores exist.
pub fn stock(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Scars::new(&mut meshes, &mut materials));
}
