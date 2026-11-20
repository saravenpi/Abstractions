//! Checks that every part of every grown look holds on: each branch, clump, cap, needle, stub, shoot,
//! flower and fruit touches or sinks into another part, and the whole plant reaches the ground, even a sloped one.

use std::collections::HashMap;

use bevy::prelude::*;

use super::tests::looks;
use super::*;
use crate::life::flora::{Form, Kind, Paint};
use crate::universe::hash::Rng;
use crate::world::solids::tests::triangles;

/// Gap in metres, at the reference height, under which two parts count as touching.
const GAP: f32 = 0.02;

/// Height in metres under which a part counts as standing on or in the ground.
const GROUND: f32 = 0.05;

/// Slope, rise over run, of the ground a plant may stand on upright: a part stands on it only where it reaches
/// below it on its downhill side.
const SLOPE: f32 = 0.5;

/// One part of a mesh: triangles joined by shared corners, and their bounds.
struct Part {
    /// Its triangles.
    tris: Vec<[Vec3; 3]>,
    /// Least corner of its bounds.
    low: Vec3,
    /// Greatest corner of its bounds.
    high: Vec3,
}

/// Splits `tris` into parts of triangles that share a corner.
fn parts(tris: &[[Vec3; 3]]) -> Vec<Part> {
    let mut root: Vec<usize> = (0..tris.len()).collect();
    let find = |root: &mut Vec<usize>, mut i: usize| {
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    };
    let mut seen: HashMap<[i32; 3], usize> = HashMap::new();
    for (i, t) in tris.iter().enumerate() {
        for p in t {
            let key = (*p * 1e4).round().as_ivec3().to_array();
            let first = *seen.entry(key).or_insert(i);
            let (a, b) = (find(&mut root, first), find(&mut root, i));
            root[a] = b;
        }
    }
    let mut out: HashMap<usize, Vec<[Vec3; 3]>> = HashMap::new();
    for (i, t) in tris.iter().enumerate() {
        out.entry(find(&mut root, i)).or_default().push(*t);
    }
    out.into_values()
        .map(|tris| {
            let corners: Vec<Vec3> = tris.iter().flatten().copied().collect();
            let low = corners.iter().fold(Vec3::MAX, |a, &b| a.min(b));
            let high = corners.iter().fold(Vec3::MIN, |a, &b| a.max(b));
            Part { tris, low, high }
        })
        .collect()
}

/// Returns the distance from `p` to triangle `t`.
fn distance(p: Vec3, [a, b, c]: [Vec3; 3]) -> f32 {
    let n = (b - a).cross(c - a);
    let edge = |u: Vec3, v: Vec3| {
        let d = v - u;
        let s = ((p - u).dot(d) / d.length_squared().max(1e-12)).clamp(0.0, 1.0);
        p.distance(u + d * s)
    };
    let side = |u: Vec3, v: Vec3| (v - u).cross(p - u).dot(n) >= 0.0;
    let inside = n.length_squared() > 1e-12 && side(a, b) && side(b, c) && side(c, a);
    match inside {
        true => (p - a).dot(n).abs() / n.length(),
        false => edge(a, b).min(edge(b, c)).min(edge(c, a)),
    }
}

/// Returns the winding number of the closed triangles `tris` round `p`: about 1 inside, 0 outside, and 0
/// for a fan seen from both sides.
fn winding(p: Vec3, tris: &[[Vec3; 3]]) -> f32 {
    let angle = |t: &[Vec3; 3]| {
        let [a, b, c] = t.map(|v| v - p);
        let (la, lb, lc) = (a.length(), b.length(), c.length());
        let below = la * lb * lc + a.dot(b) * lc + b.dot(c) * la + c.dot(a) * lb;
        2.0 * a.dot(b.cross(c)).atan2(below)
    };
    tris.iter().map(angle).sum::<f32>() / (4.0 * std::f32::consts::PI)
}

/// Returns true when part `a` touches part `b`: a point on a triangle of `a` lies within [`GAP`] of `b`
/// or inside it.
fn touch(a: &Part, b: &Part) -> bool {
    if (a.low - b.high).max_element() > GAP || (b.low - a.high).max_element() > GAP {
        return false;
    }
    let weights = (0..=4u8).flat_map(|i| (0..=4 - i).map(move |j| (i, j)));
    let points = a.tris.iter().flat_map(|&[p, q, r]| {
        weights.clone().map(move |(i, j)| {
            (p * f32::from(i) + q * f32::from(j) + r * f32::from(4 - i - j)) / 4.0
        })
    });
    points
        .filter(|p| p.cmpge(b.low - GAP).all() && p.cmple(b.high + GAP).all())
        .any(|p| b.tris.iter().any(|&t| distance(p, t) < GAP) || winding(p, &b.tris) > 0.5)
}

/// Returns the bounds of every part of `look` that neither stands on the ground nor reaches it through the
/// parts it touches.
fn floating(look: &Look) -> Vec<(Vec3, Vec3)> {
    let parts = parts(&triangles(&build(look).into_mesh()));
    let stands = |p: &Part| {
        let mut all = p.tris.iter().flatten();
        all.any(|v| v.y < GROUND - SLOPE * Vec2::new(v.x, v.z).length())
    };
    let mut held: Vec<bool> = parts.iter().map(stands).collect();
    let mut grew = true;
    while grew {
        grew = false;
        for i in 0..parts.len() {
            if !held[i]
                && (0..parts.len()).any(|j| {
                    held[j] && (touch(&parts[i], &parts[j]) || touch(&parts[j], &parts[i]))
                })
            {
                (held[i], grew) = (true, true);
            }
        }
    }
    let loose = parts.iter().zip(held).filter(|(_, h)| !h);
    loose.map(|(p, _)| (p.low, p.high)).collect()
}

/// Returns a species drawn from `rng` over every range the planet rules give.
fn species(rng: &mut Rng) -> Kind {
    let mut unit = || rng.unit() as f32;
    Kind {
        rank: (unit() * 2.99) as u8,
        size: 0.6 + 0.9 * unit(),
        spread: 0.65 + 0.8 * unit(),
        lean: 0.3 * unit(),
        branches: 2 + (unit() * 3.99) as u8,
        leaves: 3 + (unit() * 6.99) as u8,
        tint: 2.0 * unit() - 1.0,
        droop: unit(),
        ..Kind::PLAIN
    }
}

/// Returns the look of architecture `arch` at age `variant` with a species, a form and a paint drawn from
/// `rng`.
fn drawn(rng: &mut Rng, arch: u8, variant: u8) -> Look {
    let unit: [f32; 12] = std::array::from_fn(|_| rng.unit() as f32);
    let flag = |i: usize| unit[i] < 0.5;
    let colour = |i: usize, c| (unit[i] < 0.7).then_some(c);
    let form = Form {
        height: 3.0,
        layers: 1 + (unit[0] * 3.99) as u8,
        leaf: 0.15 + 1.25 * unit[1],
        needles: arch == 0 && flag(2),
        wax: flag(3),
        spines: flag(4),
        store: flag(5),
        thorns: flag(6),
        resin: flag(7),
        deciduous: flag(8),
    };
    let paint = Paint {
        leaves: colour(9, [0.05, 0.3, 0.04]),
        stem: [0.06, 0.035, 0.02],
        flowers: colour(10, [0.85, 0.05, 0.25]),
        fruit: colour(11, [0.9, 0.4, 0.02]),
    };
    let (kind, architecture) = (species(rng), arch);
    let dials = Dials {
        kind,
        architecture,
        form,
        paint,
    };
    Look { dials, variant }
}

/// No part of any grown look floats: over the swept looks and 200 drawn species per architecture and age, every
/// part reaches the ground through the parts it touches.
#[test]
fn no_part_floats() {
    let mut rng = Rng::new(0x0046_4C4F_5241);
    let species: Vec<Look> = (0..8u8)
        .flat_map(|a| (0..AGES as u8).map(move |v| (a, v)))
        .flat_map(|(a, v)| (0..200).map(move |_| (a, v)))
        .map(|(a, v)| drawn(&mut rng, a, v))
        .collect();
    let bad: Vec<String> = looks()
        .into_iter()
        .chain(species)
        .filter_map(|look| {
            let loose = floating(&look);
            (!loose.is_empty()).then(|| {
                format!(
                    "arch {} age {}: {loose:?}\n  {look:?}",
                    look.dials.architecture, look.variant
                )
            })
        })
        .collect();
    assert!(
        bad.is_empty(),
        "{} looks float parts:\n{}",
        bad.len(),
        bad.join("\n")
    );
}
