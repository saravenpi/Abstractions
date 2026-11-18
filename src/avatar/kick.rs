//! The system that throws and moves the puffs: it watches each body for the real events of
//! `dust`, reads the ground under its feet, and wakes parked puffs of the pool.
//!
//! What a body was doing on the last frame and the puffs it still owes live on that body, in
//! [`Kick`], so every body on screen raises its own dust.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Avatar;
use super::animate::Anim;
use super::dust::{Burst, From, Seen, Soil, bursts, soil};
use super::gait::foot;
use super::marks::Marks;
use super::puffs::{Puff, Puffs, Thrower, drift, recolour, throw};
use crate::player::Motion;
use crate::render::GameCamera;
use crate::space::{Anchor, up};
use crate::universe::PlanetClass;
use crate::wind::Thickness;
use crate::world::Terrain;

/// Longest frame the puffs simulate, in seconds, to survive hitches.
const MAX_DT: f32 = 0.1;

/// Most puffs a body may owe at once, so a long hitch never empties the pool in one frame.
const OWED: f32 = 24.0;

/// What one body remembers between two frames for the puffs it throws.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Kick {
    /// What it was doing on the last frame.
    pub seen: Seen,
    /// The last burst of each kind and how many of its puffs are still owed.
    pub owed: [Option<(Burst, f32)>; 5],
    /// What the puffs it owes are made of.
    pub soil: Soil,
    /// Puffs thrown so far, which seeds the next one.
    pub serial: u32,
}

impl Kick {
    /// Takes in the bursts of one frame of the state `anim` holds, on ground of `soil`.
    pub fn watch(&mut self, anim: &Anim, soil: Soil, dt: f32) {
        let input = &anim.input;
        let now = Seen::of(input, [0.0, 0.5].map(|late| planted(anim, late)));
        for (slot, found) in bursts(&self.seen, &now, input, dt).into_iter().enumerate() {
            let Some(burst) = found else {
                continue;
            };
            let left = self.owed[slot].map_or(0.0, |(_, left)| left);
            self.owed[slot] = Some((burst, (left + burst.count).min(OWED)));
        }
        (self.seen, self.soil) = (now, soil);
    }

    /// Returns the next puff owed and what it is made of, taking it off what is owed.
    pub fn due(&mut self, air: f32) -> Option<(Burst, Soil, u32)> {
        let slot = self
            .owed
            .iter_mut()
            .flatten()
            .find(|(_, left)| *left >= 1.0)?;
        slot.1 -= 1.0;
        let soil = match slot.0.from {
            From::Tips => Soil::Air,
            _ => self.soil,
        };
        self.serial = self.serial.wrapping_add(1);
        (soil != Soil::Air || air > 0.0).then_some((slot.0, soil, self.serial))
    }
}

/// Returns whether the foot that runs `late` of a stride behind the left one carries the body.
fn planted(anim: &Anim, late: f32) -> bool {
    foot(anim.clock.phase + late, anim.input.speed).planted
}

/// The world the puffs fly in.
#[derive(SystemParam)]
pub struct Ground<'w, 's> {
    /// Frame time.
    time: Res<'w, Time>,
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// The terrain of the anchored planet.
    terrain: Res<'w, Terrain>,
    /// How thick the air is, when the wind knows.
    air: Option<Res<'w, Thickness>>,
    /// The game camera, which the puffs face.
    camera: Single<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

impl Ground<'_, '_> {
    /// Returns the class of the planet and what the ground under render position `at` raises.
    fn soil(&self, at: Vec3) -> (PlanetClass, Soil) {
        let class = self.terrain.globe().record().class;
        let chart = self.terrain.locate(self.anchor.to_frame(at));
        (class, soil(class, self.terrain.surface(chart.x, chart.z)))
    }

    /// Returns how thick the air is against the air of home.
    fn air(&self) -> f32 {
        self.air.as_deref().map_or(1.0, |air| air.0)
    }
}

/// The bodies that throw puffs: the drawn body, its memory, and the player entity it hangs from.
type Bodies<'w, 's> =
    Query<'w, 's, (&'static Anim, &'static mut Kick, &'static ChildOf), With<Avatar>>;

/// Watches every body and throws the puffs it owes into parked ones.
pub fn kick(
    ground: Ground,
    mut bodies: Bodies,
    players: Query<(&Transform, &Motion), Without<Puff>>,
    (mut puffs, mut materials): (ResMut<Puffs>, ResMut<Assets<StandardMaterial>>),
    mut pool: Query<(
        &mut Puff,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let (dt, air) = (ground.time.delta_secs().min(MAX_DT), ground.air());
    let mut parked = pool.iter_mut();
    for (anim, mut kick, parent) in &mut bodies {
        let Ok((pose, motion)) = players.get(parent.parent()) else {
            continue;
        };
        let vertical = up(&ground.anchor, pose.translation);
        let (class, soil) = ground.soil(pose.translation);
        recolour(&mut puffs, class, &mut materials);
        kick.watch(anim, soil, dt);
        let thrower = Thrower {
            pose: *pose,
            velocity: Quat::from_rotation_arc(Vec3::Y, vertical) * motion.velocity,
            up: vertical,
        };
        let marks = Marks::of(&anim.pose);
        while let Some((burst, soil, serial)) = kick.due(air) {
            let Some((mut puff, mut at, mut paint)) = parked.find(|(puff, ..)| puff.life <= 0.0)
            else {
                break;
            };
            (*puff, at.translation) = throw(&burst, serial, &thrower, &marks);
            paint.set_if_neq(MeshMaterial3d(puffs.soils[soil as usize].clone()));
        }
    }
}

/// Moves the live puffs, turned to the camera, and parks the spent ones at no size.
pub fn fly(ground: Ground, mut pool: Query<(&mut Puff, &mut Transform)>) {
    let (dt, air) = (ground.time.delta_secs().min(MAX_DT), ground.air());
    let weight = ground.terrain.globe().gravity() as f32;
    for (mut puff, mut at) in &mut pool {
        if puff.life <= 0.0 {
            continue;
        }
        let vertical = up(&ground.anchor, at.translation);
        let size = drift(&mut puff, &mut at.translation, (vertical, weight, air), dt);
        at.scale = Vec3::splat(size);
        at.rotation = ground.camera.rotation();
        if puff.age >= puff.life {
            (puff.life, at.scale) = (0.0, Vec3::ZERO);
        }
    }
}
