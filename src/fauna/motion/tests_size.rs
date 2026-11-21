//! Tests of drawn sizes: the meshes of a grown body, posed at rest, are as long and as tall as its
//! mass makes it, so a 30 kg grazer stands about knee to hip high on the robot and a 400 kg one
//! twice as tall.

use bevy::math::Vec3;

use super::fixtures_life::{Grown, grown};
use super::rig::Motion;
use crate::life::body::soup;
use crate::life::{Habitat, Pattern, Plan, Reaction, Tissue, TissueKind};

/// A plain hide, whose look does not change the shape of a part.
pub(super) const HIDE: Tissue = Tissue {
    kind: TissueKind::Hide,
    hardness: 2.0,
    toughness: 0.5,
    density: 1000.0,
    elasticity: 0.5,
    heat: 330.0,
    conductivity: 0.3,
    reaction: Reaction::Inert,
    colour: [0.5, 0.4, 0.3],
    pattern: Pattern::Plain,
    gloss: 0.2,
    translucency: 0.0,
    glow: 0.0,
};

/// Returns the length along its facing and the height of the top of the drawn meshes of `g` at
/// rest, in metres, and the largest scale of any part.
fn drawn(g: &Grown) -> (f32, f32, f32) {
    let body = &g.body;
    let pose = Motion::new(body, Vec3::ZERO, Vec3::NEG_Z, 1).pose(body);
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let mut scale: f32 = 0.0;
    for (part, (at, turn, size)) in body.parts.iter().zip(pose) {
        scale = scale.max(size.max_element());
        for face in soup(part, &HIDE).faces {
            for corner in face.corners {
                let p = at + turn * (corner * size);
                (low, high) = (low.min(p), high.max(p));
            }
        }
    }
    (high.z - low.z, high.y, scale)
}

/// Returns the mean height in metres of the top of the drawn bodies of `bodies` of `mass` kg.
fn top(bodies: &[Grown], mass: f32) -> f32 {
    let tops: Vec<f32> = bodies
        .iter()
        .filter(|g| g.mass == mass)
        .map(|g| drawn(g).1)
        .collect();
    tops.iter().sum::<f32>() / tops.len().max(1) as f32
}

/// Every grown skeletal body is drawn about as long as its body length, with parts at their own
/// size, and stands taller as it grows heavier, about as the cube root of its mass.
#[test]
fn drawn_size_follows_mass() {
    let bodies: Vec<Grown> = grown(Habitat::Ground)
        .into_iter()
        .filter(|g| g.plan == Plan::Skeletal)
        .collect();
    for g in &bodies {
        let (long, high, scale) = drawn(g);
        println!(
            "{} kg archetype {}: length {:.2} drawn {long:.2}, top {high:.2}",
            g.mass, g.archetype, g.body.length
        );
        assert!(
            (0.6..=1.8).contains(&(long / g.body.length)),
            "{long} m drawn for {} m",
            g.body.length
        );
        assert!((0.5..=2.0).contains(&scale), "part scale {scale}");
    }
    let (mid, big) = (top(&bodies, 30.0), top(&bodies, 400.0));
    println!("mean top: 30 kg {mid:.2} m, 400 kg {big:.2} m");
    assert!((0.45..=1.5).contains(&mid), "{mid} m tall at 30 kg");
    assert!(
        (1.6..=3.5).contains(&(big / mid)),
        "{big} m tall at 400 kg against {mid} m"
    );
}
