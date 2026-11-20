//! One drawn animal: a root placed on the planet in its own flat frame, with one child entity per
//! body part, faded in when it comes and out when it goes.

use bevy::math::DQuat;
use bevy::prelude::*;

use super::Creature;
use super::fade::Fade;
use super::motion::{self, Motion};
use super::place::Spot;
use super::react::{Mood, Temper};
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

/// Returns the root of a body of `body` lying dead on its side at `at` in the frame `spot`, and moves
/// its standing `pose` about the middle of its torso into that root.
fn lay(spot: &Spot, body: &Body, at: Vec3, pose: &mut [motion::Placed]) -> Placed {
    let middle = at + Vec3::Y * body.height;
    for part in pose.iter_mut() {
        part.0 -= middle;
    }
    let side = (0.3 * body.height).max(0.05);
    Placed {
        pos: spot.planet(at + Vec3::Y * side),
        rot: spot.rot * DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2),
    }
}

/// Spawns the root of the animal led by `lead` of `species`, at `placed`.
fn root(commands: &mut Commands, lead: Lead, species: &Species, placed: Placed) -> Entity {
    let (herd, slot) = lead.key();
    let creature = Creature {
        species: species.id,
        herd,
        slot,
    };
    let marks = (creature, placed, DespawnOnEnter(Screen::Title));
    let shown = (Transform::default(), Visibility::default());
    commands.spawn((Name::new("Animal"), marks, shown)).id()
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
    let mut placed = Placed {
        pos: spot.origin,
        rot: spot.rot,
    };
    if matches!(birth.lead, Lead::Carcass { .. }) {
        placed = lay(&spot, &species.body, at, &mut pose);
    }
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
        fade: Fade::new(Way::In, birth.now),
    };
    commands.entity(root).insert(beast);
    root
}
