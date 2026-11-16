//! Footpaths from each door and each field to a street, kept clear of other houses and fields.

use bevy::prelude::*;

use super::fields::Field;
use super::lots::Lot;

/// Clearance kept between a path and the rows of a field.
const FIELD_PAD: f32 = 1.4;

/// Distance between the path points kept clear of plants.
const SAMPLE: f32 = 8.0;

/// Returns the distance from `p` to the segment `a..c`.
fn gap(a: Vec2, c: Vec2, p: Vec2) -> f32 {
    let k = ((p - a).dot(c - a) / (c - a).length_squared().max(1e-6)).clamp(0.0, 1.0);
    a.lerp(c, k).distance(p)
}

/// Returns true when the segment `a..c` enters the padded rectangle of `f`.
fn crosses(f: &Field, a: Vec2, c: Vec2) -> bool {
    let local = |p: Vec2| {
        let l = Quat::from_rotation_y(-f.yaw) * Vec3::new(p.x - f.pos.x, 0.0, p.y - f.pos.y);
        Vec2::new(l.x, l.z)
    };
    let (p, d, h) = (local(a), local(c) - local(a), f.half + FIELD_PAD);
    let mut span = (0.0f32, 1.0f32);
    for (p, d, h) in [(p.x, d.x, h.x), (p.y, d.y, h.y)] {
        let d = if d.abs() < 1e-6 { 1e-6 } else { d };
        let (t0, t1) = ((-h - p) / d, (h - p) / d);
        span = (span.0.max(t0.min(t1)), span.1.min(t0.max(t1)));
    }
    span.0 <= span.1
}

/// Returns true when the segment `seg` misses every lot circle and field, except number `own` of lots then fields.
fn clear(lots: &[Lot], fields: &[Field], own: usize, seg: (Vec2, Vec2)) -> bool {
    let (a, c) = seg;
    let off_lots = |(i, l): (usize, &Lot)| i == own || gap(a, c, l.pos) >= l.house.radius();
    let off_fields = |(i, f): (usize, &Field)| i + lots.len() == own || !crosses(f, a, c);
    lots.iter().enumerate().all(off_lots) && fields.iter().enumerate().all(off_fields)
}

/// Returns the point in front of the door of `l` where its path starts.
fn door(l: &Lot) -> Vec2 {
    let d = l.house.dims();
    l.pos + Vec2::new(l.yaw.sin(), l.yaw.cos()) * (d.y + d.z + 2.4)
}

/// Lists the footpaths: one per lot from its door to a street, in lot order, then one per field that has a
/// clear way. Each goes to the nearest street sample whose segment is clear; a door falls back to the nearest.
pub fn paths_for(streets: &[Vec<Vec2>], lots: &[Lot], fields: &[Field]) -> Vec<(Vec2, Vec2)> {
    let mut samples: Vec<Vec2> = streets.iter().flatten().copied().collect();
    let starts = lots.iter().map(door).chain(fields.iter().map(|f| f.pos));
    let mut out = Vec::new();
    for (i, from) in starts.enumerate() {
        let trim = if i < lots.len() { 0.0 } else { 6.0 };
        samples.sort_by(|a, b| a.distance(from).total_cmp(&b.distance(from)));
        let reach = samples.first().map_or(0.0, |s| s.distance(from)) * 1.5 + 6.0;
        let seg = |to: &Vec2| (from + (*to - from).normalize_or_zero() * trim, *to);
        let near = samples.iter().take_while(|s| s.distance(from) <= reach);
        let free = near.map(seg).find(|s| clear(lots, fields, i, *s));
        let nearest = samples.first().filter(|_| trim == 0.0).map(seg);
        out.extend(free.or(nearest));
    }
    out
}

/// Returns points every 8 m along `paths`, for the clearing kept free of plants.
pub fn samples(paths: &[(Vec2, Vec2)]) -> impl Iterator<Item = Vec2> + '_ {
    paths.iter().flat_map(|(a, c)| {
        let n = (a.distance(*c) / SAMPLE).ceil().max(1.0) as u32;
        (0..n).map(move |i| a.lerp(*c, i as f32 / n as f32))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::structures::village::fields::Crop;

    /// A path through a field is seen, whatever the yaw of the field, and a path beside it is not.
    #[test]
    fn field_rectangles_block_paths() {
        for yaw in [0.0, 0.7, 2.9] {
            let f = Field {
                pos: Vec2::new(10.0, -4.0),
                yaw,
                half: Vec2::new(5.0, 3.0),
                crop: Crop::Wheat,
            };
            let along = Quat::from_rotation_y(yaw) * Vec3::X;
            let (along, across) = (Vec2::new(along.x, along.z), Vec2::new(-along.z, along.x));
            let through = (f.pos - across * 9.0, f.pos + across * 9.0);
            let beside = (through.0 + along * 7.0, through.1 + along * 7.0);
            assert!(crosses(&f, through.0, through.1), "yaw {yaw}");
            assert!(!crosses(&f, beside.0, beside.1), "yaw {yaw}");
            assert!(
                !crosses(&f, through.0, through.0 - across * 5.0),
                "yaw {yaw}"
            );
        }
    }
}
