//! One drawn animal: a root placed on the planet in its own flat frame, with one child entity per
//! body part, faded in when it comes and out when it goes.

use bevy::prelude::*;

use super::Creature;
use super::attack::{BossMind, Lain};
use super::fade::Fade;
use super::motion::{self, Motion};
use super::place::Spot;
use super::react::{Mood, Temper};
use super::senses::Wits;
use super::want::Lead;
use crate::life::Species;
use crate::life::body::Body;
use crate::screens::Screen;
use crate::space::Placed;
use crate::world::dither::{Way, range};

/// The state of one drawn animal.
#[derive(Component, Debug, Clone)]
pub struct Beast {
    /// What leads it.
    pub lead: Lead,
    /// Index of its species in the planet's life.
    pub index: u16,
    /// The flat frame it moves in.
    pub spot: Spot,
    /// Its motion in that frame.
    pub motion: Motion,
    /// One entity per body part, `None` for a part with no mesh.
    pub parts: Vec<Option<Entity>>,
    /// Its alarm.
    pub mood: Mood,
    /// How its species reacts.
    pub temper: Temper,
    /// What it senses and its health.
    pub wits: Wits,
    /// Its fade in or out.
    pub fade: Fade,
}

/// Marks a drawn body part of an animal.
#[derive(Component, Debug, Clone, Copy)]
pub struct Part;

/// Where and how an animal comes into the world.
pub struct Birth<'a> {
    /// What leads it.
    pub lead: Lead,
    /// Its species and its index in the planet's life.
    pub species: (&'a Species, u16),
    /// Its flat frame, and its place and facing in that frame.
    pub at: (Spot, Vec3, Vec3),
    /// Game seconds now, when its fade starts.
    pub now: f32,
}

/// Returns the root of a body of `body` lying dead on its side at `at` in the frame `spot`, rolled over
/// about the way it faces, `facing`, and how it lies there, so it is laid on the drawn ground; moves its
/// standing `pose` about the middle of its torso into that root. Rolled about any other axis it stood
/// on its head or its tail, half of it in the air.
pub(super) fn lay(
    spot: &Spot,
    body: &Body,
    (at, facing): (Vec3, Vec3),
    pose: &mut [motion::Placed],
) -> (Placed, Option<Lain>) {
    let middle = at + Vec3::Y * body.height;
    for part in pose.iter_mut() {
        part.0 -= middle;
    }
    let side = (0.3 * body.height).max(0.05);
    let along = facing.with_y(0.0).normalize_or(Vec3::NEG_Z);
    let turn = Quat::from_axis_angle(along, std::f32::consts::FRAC_PI_2);
    let lain = Lain {
        pivot: Vec3::ZERO,
        turn,
        at: Vec2::new(at.x, at.z),
        middle: Vec2::ZERO,
        side,
        reach: (0.5 * body.length).max(0.05),
    };
    let placed = Placed {
        pos: spot.planet(at + Vec3::Y * side),
        rot: (spot.rot * turn.as_dquat()).normalize(),
    };
    (placed, Some(lain))
}

/// Spawns the root of the animal led by `lead` of `species`, at `placed`, lying as it says when it is a
/// carcass.
fn root(
    commands: &mut Commands,
    lead: Lead,
    species: &Species,
    (placed, lain): (Placed, Option<Lain>),
) -> Entity {
    let (herd, slot) = lead.key();
    let creature = Creature {
        species: species.id,
        herd,
        slot,
    };
    let marks = (creature, placed, DespawnOnEnter(Screen::Title));
    let shown = (Transform::default(), Visibility::default());
    let mut root = commands.spawn((Name::new("Animal"), marks, shown));
    if let Some(lain) = lain {
        root.insert(lain);
    }
    root.id()
}

/// Spawns one child of `root` per part mesh of `meshes` drawn with its material of `skins` at its
/// place in `pose`, dithered fully out, and returns them in the order of the parts.
fn limbs(
    commands: &mut Commands,
    root: Entity,
    (meshes, skins): (&[Option<Handle<Mesh>>], &[Handle<StandardMaterial>]),
    pose: Vec<motion::Placed>,
) -> Vec<Option<Entity>> {
    let part = |((mesh, skin), (at, turn, scale)): (
        (&Option<Handle<Mesh>>, &Handle<StandardMaterial>),
        motion::Placed,
    )| {
        let look = (Mesh3d(mesh.clone()?), MeshMaterial3d(skin.clone()));
        let place = Transform {
            translation: at,
            rotation: turn,
            scale,
        };
        let marks = (Part, range(Way::In, 0), ChildOf(root));
        Some(commands.spawn((look, place, marks)).id())
    };
    meshes.iter().zip(skins).zip(pose).map(part).collect()
}

/// Spawns the animal of `birth` with the part meshes `meshes` and the part materials `skins`,
/// fading in.
pub fn spawn(
    commands: &mut Commands,
    birth: Birth,
    meshes: &[Option<Handle<Mesh>>],
    skins: &[Handle<StandardMaterial>],
) -> Entity {
    let (species, index) = birth.species;
    let (spot, at, facing) = birth.at;
    let seed = birth.lead.key().0 ^ u64::from(birth.lead.key().1);
    let motion = Motion::new(&species.body, at, facing, seed);
    let mut pose = motion.pose(&species.body);
    let standing = Placed {
        pos: spot.origin,
        rot: spot.rot,
    };
    let placed = match birth.lead {
        Lead::Carcass { .. } => lay(&spot, &species.body, (at, facing), &mut pose),
        _ => (standing, None),
    };
    let root = root(commands, birth.lead, species, placed);
    let parts = limbs(commands, root, (meshes, skins), pose);
    let beast = Beast {
        lead: birth.lead,
        index,
        spot,
        motion,
        parts,
        mood: Mood::default(),
        temper: Temper::of(species),
        wits: Wits::of(species),
        fade: Fade::new(Way::In, birth.now),
    };
    commands.entity(root).insert(beast);
    if species.boss {
        commands.entity(root).insert(BossMind::new());
    }
    root
}
