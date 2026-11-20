//! Hand-built bodies that do not walk, for the motion tests: a snake, a jelly, a slime, a
//! crystal, a flyer and a fish.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};

use super::fixtures::{chain, leg, part, tail, torso};
use crate::life::body::{Body, Knee, Locomotion, PartKind, Wings};

/// Returns a snake 1.8 m long: a head and eight spine segments trailing behind it.
pub fn snake() -> Body {
    let mut body = torso(
        PartKind::Head,
        Locomotion::Slither,
        (1.8, 0.08),
        Vec3::new(0.1, 0.08, 0.2),
    );
    let back = Quat::from_rotation_x(-FRAC_PI_2);
    body.spine = Some(chain(
        &mut body,
        PartKind::Segment,
        (0, Vec3::new(0.0, 0.0, 0.1), back),
        (8, 0.2),
        0.5,
    ));
    body
}

/// Returns a jelly: a bell half a metre across with four tentacles hanging under it.
pub fn jelly() -> Body {
    let mut body = torso(
        PartKind::Bell,
        Locomotion::Pulse,
        (0.5, 0.5),
        Vec3::new(0.5, 0.3, 0.5),
    );
    for i in 0..4 {
        let angle = i as f32 * FRAC_PI_2;
        let at = Vec3::new(0.18 * angle.cos(), -0.1, 0.18 * angle.sin());
        let tentacle = chain(
            &mut body,
            PartKind::Tentacle,
            (0, at, Quat::IDENTITY),
            (3, 0.15),
            0.7,
        );
        body.tentacles.push(tentacle);
    }
    body
}

/// Returns a slime: one blob 0.4 m across.
pub fn slime() -> Body {
    torso(
        PartKind::Blob,
        Locomotion::Ooze,
        (0.4, 0.2),
        Vec3::new(0.4, 0.3, 0.4),
    )
}

/// Returns a crystal walker: one block 0.6 m across that tumbles.
pub fn crystal() -> Body {
    torso(
        PartKind::Crystal,
        Locomotion::Tumble,
        (0.6, 0.3),
        Vec3::splat(0.6),
    )
}

/// Adds to `body` a pair of wings `span` wide flapping `rate` times a second.
fn wings(body: &mut Body, span: f32, rate: f32) {
    let first = body.parts.len() as u8;
    for side in [-1i8, 1] {
        let at = Vec3::new(0.08 * f32::from(side), 0.0, 0.0);
        let size = Vec3::new(0.5 * span, 0.02, 0.25);
        let mut wing = part(PartKind::Wing, Some(0), (at, Quat::IDENTITY), size);
        wing.side = side;
        body.parts.push(wing);
    }
    body.wings = Some(Wings {
        parts: [first, first + 1],
        span,
        rate,
    });
}

/// Returns a flyer with a 1.2 m span, a short tail and tucked legs.
pub fn flyer() -> Body {
    let size = Vec3::new(0.15, 0.15, 0.4);
    let mut body = torso(PartKind::Torso, Locomotion::Fly, (0.5, 0.2), size);
    wings(&mut body, 1.2, 3.0);
    tail(&mut body, Vec3::new(0.0, 0.0, 0.2), (2, 0.1), 0.4);
    for side in [-1i8, 1] {
        let x = 0.05 * f32::from(side);
        let ends = (Vec3::new(x, 0.15, 0.0), Vec3::new(x, 0.0, 0.0));
        leg(&mut body, ends, &[0.1, 0.1], Knee::Back, (side, 0));
    }
    body
}

/// Returns a fish 0.8 m long with a three-segment tail.
pub fn fish() -> Body {
    let size = Vec3::new(0.15, 0.2, 0.4);
    let mut body = torso(PartKind::Torso, Locomotion::Swim, (0.8, 0.2), size);
    tail(&mut body, Vec3::new(0.0, 0.0, 0.2), (3, 0.15), 0.5);
    body
}
