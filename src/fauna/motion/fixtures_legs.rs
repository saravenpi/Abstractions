//! Hand-built walkers for the motion tests: a bird-kneed biped and sprawling crawlers.

use bevy::math::{Quat, Vec3};

use super::fixtures::{leg, part, torso};
use crate::life::body::{Body, Knee, Locomotion, PartKind};

/// Returns a bird-kneed biped with three-bone legs and hips 0.6 m high.
pub fn biped() -> Body {
    let mut body = torso(
        PartKind::Torso,
        Locomotion::Walk,
        (0.6, 0.6),
        Vec3::new(0.3, 0.3, 0.5),
    );
    for side in [-1i8, 1] {
        let x = 0.1 * f32::from(side);
        leg(
            &mut body,
            (Vec3::new(x, 0.6, 0.0), Vec3::new(x, 0.0, 0.0)),
            &[0.25, 0.3, 0.15],
            Knee::Back,
            (side, 0),
        );
    }
    body.parts.push(part(
        PartKind::Head,
        Some(0),
        (Vec3::new(0.0, 0.3, -0.25), Quat::IDENTITY),
        Vec3::splat(0.12),
    ));
    body
}

/// Returns a sprawling walker with `ranks` pairs of legs spread along a body `length` long.
pub fn crawler(ranks: u8, length: f32) -> Body {
    let height = 0.3;
    let mut body = torso(
        PartKind::Torso,
        Locomotion::Walk,
        (length, height),
        Vec3::new(0.3, 0.2, length),
    );
    for rank in 0..ranks {
        let z = length * (f32::from(rank) / f32::from(ranks.max(2) - 1) - 0.5) * 0.8;
        for side in [-1i8, 1] {
            let x = f32::from(side);
            let ends = (Vec3::new(0.15 * x, height, z), Vec3::new(0.45 * x, 0.0, z));
            leg(&mut body, ends, &[0.3, 0.35], Knee::Out, (side, rank));
        }
    }
    let mut antenna = part(
        PartKind::Antenna,
        Some(0),
        (Vec3::new(0.05, 0.1, -0.5 * length), Quat::IDENTITY),
        Vec3::splat(0.03),
    );
    antenna.side = 1;
    body.parts.push(antenna);
    body
}
