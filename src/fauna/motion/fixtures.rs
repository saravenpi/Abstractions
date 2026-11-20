//! Hand-built bodies for the motion tests: the part, leg and chain builders, and a quadruped
//! with a neck of any length.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};

use crate::life::body::{Body, Chain, Knee, Leg, Locomotion, Part, PartKind, ShapeKind};

/// Returns a part of `kind` hanging from `parent` at `socket`, turned by `rest`, `size` big.
pub fn part(kind: PartKind, parent: Option<u8>, (socket, rest): (Vec3, Quat), size: Vec3) -> Part {
    Part {
        kind,
        parent,
        socket,
        rest,
        shape: ShapeKind::Block,
        size,
        taper: 1.0,
        tissue: 0,
        side: 0,
    }
}

/// Returns an empty body of `locomotion`, `length` long with its hips `height` high, its torso `size` big.
pub fn torso(
    kind: PartKind,
    locomotion: Locomotion,
    (length, height): (f32, f32),
    size: Vec3,
) -> Body {
    Body {
        parts: vec![part(kind, None, (Vec3::Y * height, Quat::IDENTITY), size)],
        legs: Vec::new(),
        spine: None,
        neck: None,
        tail: None,
        tentacles: Vec::new(),
        wings: None,
        locomotion,
        length,
        height,
        features: Vec::new(),
    }
}

/// Adds to `body` a leg hanging from its torso at `hip` with bones `lengths`, bending `knee`,
/// on `side` in `rank`, whose foot rests at `foot` in the body frame.
pub fn leg(
    body: &mut Body,
    (hip, foot): (Vec3, Vec3),
    lengths: &[f32],
    knee: Knee,
    (side, rank): (i8, u8),
) {
    let mut bones = Vec::new();
    let kinds = [PartKind::Thigh, PartKind::Shin, PartKind::Foot];
    let mut socket = hip - body.parts[0].socket;
    let mut parent = 0u8;
    for (i, &length) in lengths.iter().enumerate() {
        let mut bone = part(
            kinds[i.min(2)],
            Some(parent),
            (socket, Quat::IDENTITY),
            Vec3::new(0.06, length, 0.06),
        );
        bone.side = side;
        parent = body.parts.len() as u8;
        body.parts.push(bone);
        bones.push(parent);
        socket = Vec3::NEG_Y * length;
    }
    body.legs.push(Leg {
        bones,
        lengths: lengths.to_vec(),
        rest: foot,
        knee,
        side,
        rank,
    });
}

/// Returns a chain of `count` parts of `kind`, each `length` long, from `parent` at `socket`,
/// the first turned by `rest`, bending at most `limit` per joint; the parts are added to `body`.
pub fn chain(
    body: &mut Body,
    kind: PartKind,
    (parent, socket, rest): (u8, Vec3, Quat),
    (count, length): (usize, f32),
    limit: f32,
) -> Chain {
    let mut parts = Vec::new();
    let (mut parent, mut socket, mut turn) = (parent, socket, rest);
    for _ in 0..count {
        body.parts.push(part(
            kind,
            Some(parent),
            (socket, turn),
            Vec3::new(0.1, length, 0.1),
        ));
        parent = (body.parts.len() - 1) as u8;
        parts.push(parent);
        (socket, turn) = (Vec3::NEG_Y * length, Quat::IDENTITY);
    }
    Chain {
        parts,
        lengths: vec![length; count],
        limit,
    }
}

/// Adds to `body` a tail of `count` parts `length` long hanging back from the torso at `at`.
pub fn tail(body: &mut Body, at: Vec3, (count, length): (usize, f32), limit: f32) {
    let back = Quat::from_rotation_x(-FRAC_PI_2);
    let tail = chain(body, PartKind::Tail, (0, at, back), (count, length), limit);
    body.tail = Some(tail);
}

/// Adds to `body` a neck of `count` parts `length` long rising forward from the torso at `at`,
/// with a head on its tip.
pub fn neck(body: &mut Body, at: Vec3, (count, length): (usize, f32)) {
    let up = Quat::from_rotation_x(2.4);
    let neck = chain(body, PartKind::Neck, (0, at, up), (count, length), 0.5);
    let tip = neck.parts[neck.parts.len() - 1];
    let head = (Vec3::NEG_Y * length, up.inverse());
    body.parts
        .push(part(PartKind::Head, Some(tip), head, Vec3::splat(0.25)));
    body.neck = Some(neck);
}

/// Returns a quadruped 1.8 m long with hips 1 m high, a neck with a head and a tail; its neck
/// has `necks` segments `length` long.
pub fn quadruped_with(necks: usize, length: f32) -> Body {
    let size = Vec3::new(0.5, 0.5, 1.4);
    let mut body = torso(PartKind::Torso, Locomotion::Walk, (1.8, 1.0), size);
    for (rank, z, knee) in [(0u8, -0.55, Knee::Back), (1, 0.55, Knee::Forward)] {
        for side in [-1i8, 1] {
            let x = 0.2 * f32::from(side);
            let ends = (Vec3::new(x, 1.0, z), Vec3::new(x, 0.0, z));
            leg(&mut body, ends, &[0.55, 0.55], knee, (side, rank));
        }
    }
    neck(&mut body, Vec3::new(0.0, 0.2, -0.7), (necks, length));
    tail(&mut body, Vec3::new(0.0, 0.1, 0.7), (3, 0.25), 0.6);
    body
}
