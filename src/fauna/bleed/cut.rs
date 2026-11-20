//! Cutting: a blow that lands on an animal may open it, and then the blood sprays at once; one that
//! does not cut may knock chips off a hard layer.

use bevy::camera::primitives::Aabb;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::chips::chips;
use super::open::opened;
use super::sound::impact;
use super::throw::{Jet, burst, scatter};
use super::tune::CHIP;
use super::wound::{Circuit, Cut, Wound};
use crate::audio::{Body, Impact};
use crate::avatar::strike::{Hit, Outcome};
use crate::effects::Fling;
use crate::fauna::Part;
use crate::fauna::attack::Dead;
use crate::fauna::beast::Beast;
use crate::life::blood::tune::VENOUS;
use crate::life::blood::{species as blood_of, speed};
use crate::life::{Life, Species};
use crate::settings::{Persistent, Settings};
use crate::world::Terrain;

/// What a cut reads and writes besides the animals.
#[derive(SystemParam)]
pub struct Strikes<'w, 's> {
    commands: Commands<'w, 's>,
    terrain: Res<'w, Terrain>,
    settings: Res<'w, Persistent<Settings>>,
    time: Res<'w, Time>,
    flings: MessageWriter<'w, Fling>,
    sounds: MessageWriter<'w, Impact>,
    parts: Query<'w, 's, (&'static GlobalTransform, &'static Aabb), With<Part>>,
    roots: Query<'w, 's, &'static GlobalTransform, With<Beast>>,
}

/// Returns the unit direction the surface of the part `hit` struck faces at the point of contact: from
/// the middle of the part to the point, or against the blow when the part cannot be read.
fn normal(parts: &Query<(&GlobalTransform, &Aabb), With<Part>>, hit: &Hit) -> Vec3 {
    let middle = parts
        .get(hit.part)
        .map(|(placed, aabb)| placed.transform_point(Vec3::from(aabb.center)))
        .unwrap_or(hit.at - hit.along);
    (hit.at - middle).normalize_or(-hit.along)
}

/// Throws the chips of `hit` off its part: deflected off the surface, widely scattered.
fn flake(s: &mut Strikes, hit: &Hit, (normal, species, key): (Vec3, (&Species, usize), u32)) {
    let glanced = hit.outcome == Outcome::Deflected;
    let Some(c) = chips(species.0, species.1, &hit.blow, glanced) else {
        return;
    };
    let bounce = hit.along.reflect(normal);
    for n in 0..c.count as u32 {
        s.flings.write(Fling {
            at: hit.at,
            velocity: scatter(bounce, 3.0, key.wrapping_add(n * 53)) * c.speed,
            radius: CHIP,
            colour: c.colour,
            wet: false,
            tag: None,
        });
    }
}

/// Returns the wound a blow `hit` made, in the frame of the animal whose root is `root`.
fn wound_of(
    root: &GlobalTransform,
    hit: &Hit,
    (index, normal): (usize, Vec3),
    open: super::open::Opening,
) -> Cut {
    let inverse = root.affine().inverse();
    Cut {
        part: index as u8,
        at: inverse.transform_point3(hit.at),
        normal: inverse.transform_vector3(normal).normalize_or(Vec3::Y),
        width: open.width,
        depth: open.depth,
        major: open.major,
        age: 0.0,
        owed: 0.0,
    }
}

/// Returns the cut that `hit` made in the animal `species` of index `beast` whose root is `root`, and
/// the jet its blood bursts out in: along the blow and the surface normal at the speed its pressure
/// gives. A blow that opens nothing, a bruise, makes none, nor does one that glanced off or a body with
/// no blood.
fn opening(
    (life, gravity): (&Life, f32),
    (species, beast): (&Species, u16),
    (hit, root): (&Hit, &GlobalTransform),
    (index, normal): (usize, Vec3),
) -> Option<(Jet, Cut)> {
    if hit.outcome != Outcome::Clean {
        return None;
    }
    let blood = blood_of(life, species)?;
    let open = opened(species, index, &hit.blow)?;
    let circuit = Circuit::of(species, &blood, gravity);
    let fast = speed(if open.major { circuit.pressure } else { VENOUS });
    let dir = (hit.along * 0.6 + normal * 0.8).normalize_or(normal);
    let colour = blood.drop(open.major);
    let jet = Jet {
        at: hit.at,
        dir,
        speed: fast,
        colour,
        species: beast,
    };
    Some((jet, wound_of(root, hit, (index, normal), open)))
}

/// Sends the sound of `hit` on part `index` of `species`, `cut` when it opened it.
fn ring(s: &mut Strikes, hit: &Hit, (species, index): (&Species, usize), cut: bool) {
    let tool = Body {
        stiffness: hit.tip.stiffness,
        mass: hit.tip.mass,
    };
    let heard = impact(species, index, (hit.at, hit.blow.energy, cut), tool);
    s.sounds.write(Impact {
        deflected: hit.outcome == Outcome::Deflected,
        ..heard
    });
}

/// Adds `cut` to the wound of `victim`, or gives it its first.
fn record(commands: &mut Commands, victim: Entity, wound: Option<Mut<Wound>>, cut: Cut) {
    match wound {
        Some(mut w) => w.add(cut),
        None => {
            let mut w = Wound::default();
            w.add(cut);
            commands.entity(victim).insert(w);
        }
    }
}

/// Lets every blow that lands on an animal knock chips off a hard layer it did not cut, or open it
/// and spray.
pub fn cut(
    mut s: Strikes,
    mut hits: MessageReader<Hit>,
    mut beasts: Query<(&Beast, Option<&mut Wound>), Without<Dead>>,
) {
    let globe = s.terrain.globe().clone();
    let Some(life) = globe.life() else { return };
    let key = (s.time.elapsed_secs() * 1000.0) as u32;
    for hit in hits.read() {
        let Ok((beast, wound)) = beasts.get_mut(hit.victim) else {
            continue;
        };
        let known = life.species.get(usize::from(beast.index));
        let index = beast.parts.iter().position(|p| *p == Some(hit.part));
        let (Some(species), Some(index), Ok(root)) = (known, index, s.roots.get(hit.victim)) else {
            continue;
        };
        let root = *root;
        let normal = normal(&s.parts, hit);
        flake(&mut s, hit, (normal, (species, index), key));
        let gravity = (life, globe.gravity() as f32);
        let made = opening(
            gravity,
            (species, beast.index),
            (hit, &root),
            (index, normal),
        );
        ring(&mut s, hit, (species, index), made.is_some());
        let Some((jet, cut)) = made else { continue };
        if s.settings.gameplay.blood {
            burst(&mut s.flings, &jet, (cut.width, cut.depth), key);
        }
        record(&mut s.commands, hit.victim, wound, cut);
    }
}
