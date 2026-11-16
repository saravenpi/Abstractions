//! Lots along the streets: ground checks, spacing and the choice of house for each.

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;

pub use super::kinds::House;
use super::kinds::pick;
use super::layout::PLAZA;
use super::style::{Flavor, Style};
use super::trace::{walkable, yaw_of};
use crate::world::Terrain;

/// Half width of a street strip.
pub const ROAD_HALF: f32 = 1.7;

/// Gap kept between a street strip and the front of a lot.
const GAP: f32 = 2.0;

/// One building plot.
#[derive(Debug, Clone, Copy)]
pub struct Lot {
    /// World position of the house center.
    pub pos: Vec2,
    /// Yaw whose local +z faces the street.
    pub yaw: f32,
    /// World height of the floor.
    pub floor: f32,
    /// How far the foundation reaches below the floor.
    pub depth: f32,
    /// What stands on it.
    pub house: House,
    /// Random bits for small variations.
    pub variant: u32,
}

/// A place along a street where a lot may go.
#[derive(Debug, Clone, Copy)]
pub struct Spot {
    /// Index of the street.
    pub street: usize,
    /// Index of the street sample.
    pub index: usize,
    /// Which side of the street, -1 or 1.
    pub side: f32,
}

/// Returns the ground heights under the corners and front of a house, or None when any is wet or rocky.
fn sample_heights(terrain: &Terrain, house: House, pos: Vec2, yaw: f32) -> Option<Vec<f32>> {
    let d = house.dims();
    let rot = Quat::from_rotation_y(yaw);
    let local = [
        (0.0, 0.0),
        (-1.0, -1.0),
        (1.0, -1.0),
        (-1.0, 1.0),
        (1.0, 1.0),
        (0.0, 1.0),
    ];
    local
        .iter()
        .map(|(sx, sz)| {
            let off = rot
                * Vec3::new(
                    sx * (d.x + 0.6),
                    0.0,
                    sz * (d.y + d.z + 0.6) + sz.max(0.0) * 1.5,
                );
            let p = pos + Vec2::new(off.x, off.z);
            walkable(terrain, p).then(|| terrain.height(p.x, p.y))
        })
        .collect()
}

/// Returns the floor height and foundation depth of `house` at `pos`, if the ground is gentle and dry.
fn fit(terrain: &Terrain, style: &Style, house: House, pos: Vec2, yaw: f32) -> Option<(f32, f32)> {
    let heights = sample_heights(terrain, house, pos, yaw)?;
    let (lo, hi) = heights
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    let lift = if style.flavor == Flavor::Stilt {
        1.1
    } else {
        0.3
    };
    (hi - lo <= 1.5).then(|| (hi + lift, (hi + lift - lo + 0.7).max(1.4)))
}

/// What the lot search needs to know about the village.
pub struct Plan {
    /// Look of the village.
    pub style: Style,
    /// World position of the plaza center.
    pub center: Vec2,
    /// Number of buildings wanted.
    pub want: usize,
}

/// Lists the places along every street where a lot may go, nearest the plaza first.
fn spots_for(rng: &mut ChaCha8Rng, streets: &[Vec<Vec2>]) -> Vec<Spot> {
    let mut spots = Vec::new();
    for (street, pts) in streets.iter().enumerate() {
        let mut index = 4;
        while index + 1 < pts.len() {
            spots.extend([-1.0, 1.0].map(|side| Spot {
                street,
                index,
                side,
            }));
            index += rng.random_range(4..=5);
        }
    }
    spots.sort_by_key(|s| (s.index, s.street));
    spots
}

/// Returns the position and facing of a `house` placed at `spot`.
fn plot(streets: &[Vec<Vec2>], spot: &Spot, house: House) -> (Vec2, f32) {
    let pts = &streets[spot.street];
    let dir = (pts[spot.index + 1] - pts[spot.index]).normalize();
    let normal = Vec2::new(-dir.y, dir.x) * spot.side;
    let d = house.dims();
    (
        pts[spot.index] + normal * (ROAD_HALF + GAP + d.y + d.z),
        yaw_of(-normal),
    )
}

/// Returns true when a circle `at = (pos, radius)` keeps clear of other lots, the plaza and other streets.
fn clear_of(streets: &[Vec<Vec2>], lots: &[Lot], own: usize, at: (Vec2, f32, Vec2)) -> bool {
    let (pos, rad, center) = at;
    lots.iter()
        .all(|l| l.pos.distance(pos) > rad + l.house.radius())
        && pos.distance(center) > PLAZA + rad
        && streets
            .iter()
            .enumerate()
            .all(|(i, s)| i == own || s.iter().all(|p| p.distance(pos) > rad + ROAD_HALF))
}

/// Returns the lot for `house` at `spot`, if it keeps clear of the others and the ground fits.
fn lot_at(
    terrain: &Terrain,
    rng: &mut ChaCha8Rng,
    streets: &[Vec<Vec2>],
    plan: &Plan,
    at: (&Spot, &[Lot], House),
) -> Option<Lot> {
    let (spot, lots, house) = at;
    let (pos, yaw) = plot(streets, spot, house);
    if !clear_of(
        streets,
        lots,
        spot.street,
        (pos, house.radius(), plan.center),
    ) {
        return None;
    }
    let (floor, depth) = fit(terrain, &plan.style, house, pos, yaw)?;
    let variant = rng.random_range(0..u32::MAX);
    Some(Lot {
        pos,
        yaw,
        floor,
        depth,
        house,
        variant,
    })
}

/// Chooses the lots of a village: the hall first, then weighted houses, up to `plan.want` buildings.
pub fn place(
    terrain: &Terrain,
    rng: &mut ChaCha8Rng,
    streets: &[Vec<Vec2>],
    plan: &Plan,
) -> Vec<Lot> {
    let mut lots: Vec<Lot> = Vec::new();
    for spot in spots_for(rng, streets) {
        if lots.len() >= plan.want {
            break;
        }
        let used: Vec<House> = lots.iter().map(|l| l.house).collect();
        let house = if lots.is_empty() {
            House::Hall
        } else {
            pick(rng, &used)
        };
        if let Some(lot) = lot_at(terrain, rng, streets, plan, (&spot, &lots, house)) {
            lots.push(lot);
        }
    }
    lots
}
