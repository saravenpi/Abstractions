//! The tiny home station: a canopy on four posts over a charging dock, a crate and the beacon mast.

use bevy::math::Affine3A;
use bevy::prelude::*;

use super::draft::Draft;
use super::glow::Glow;
use super::lot::Lot;
use super::paint::{CRATE, DARK, HATCH, HULL, PANEL, TRIM};
use super::tune::STATION;

/// Where the posts of the canopy stand, right and back of the centre of the station.
pub const POSTS: [Vec2; 4] = [
    Vec2::new(-1.8, -1.4),
    Vec2::new(1.8, -1.4),
    Vec2::new(1.8, 1.4),
    Vec2::new(-1.8, 1.4),
];

/// Where the cables from the container plug into the dock.
pub const PLUG: Vec3 = Vec3::new(1.5, 0.6, -1.1);

/// Height in metres of the underside of the canopy, of the bench top and of the beacon.
const HEIGHTS: (f32, f32, f32) = (2.5, 0.85, 5.0);

/// Where the legs of the bench stand.
const BENCH: [Vec2; 4] = [
    Vec2::new(-0.5, -1.35),
    Vec2::new(1.4, -1.35),
    Vec2::new(1.4, -0.85),
    Vec2::new(-0.5, -0.85),
];

/// Centre of the storage crate and the length of its sides, in metres.
const BOX: (Vec2, f32) = (Vec2::new(-1.3, 0.6), 0.8);

/// Where the beacon mast stands.
const MAST: Vec2 = Vec2::new(-2.5, -1.5);

/// Returns the pose of the station on `lot`, in the frame of the base.
pub fn pose(lot: &Lot) -> Affine3A {
    lot.stand(STATION, &POSTS, 0.0)
}

/// Adds the four posts and the canopy with its dark top.
fn shelter(draft: &mut Draft) {
    let top = HEIGHTS.0;
    for post in POSTS {
        let (low, half) = (draft.ground(post), Vec2::splat(0.06));
        draft.leg(post, top, half.x, TRIM);
        draft.block(
            (post - half).extend(low).xzy(),
            (post + half).extend(top).xzy(),
        );
    }
    draft.deck(
        Vec3::new(-2.2, top, -1.8),
        Vec3::new(2.2, top + 0.1, 1.8),
        HULL,
    );
    draft.draw(|soup| {
        soup.slab(
            Vec3::new(-1.9, top + 0.1, -1.5),
            Vec3::new(1.9, top + 0.13, 1.5),
            PANEL,
        );
    });
}

/// Adds the bench of the charging dock on its four legs, and makes it solid.
fn bench(draft: &mut Draft) {
    let top = HEIGHTS.1;
    for foot in BENCH {
        draft.leg(foot, top, 0.035, DARK);
    }
    draft.deck(
        Vec3::new(-0.6, top, -1.45),
        Vec3::new(1.5, top + 0.07, -0.75),
        TRIM,
    );
    let low = draft.lowest(&BENCH);
    draft.block(
        Vec3::new(-0.6, low, -1.45),
        Vec3::new(1.5, top + 0.07, -0.75),
    );
}

/// Adds what makes the bench a charging dock: a cradle for each of the two robots, a back
/// panel and a soft light under its shelf.
fn dock(draft: &mut Draft) {
    let bed = HEIGHTS.1 + 0.07;
    draft.draw(|soup| {
        soup.slab(
            Vec3::new(-0.6, bed, -1.45),
            Vec3::new(1.5, 1.7, -1.38),
            TRIM,
        );
        soup.slab(
            Vec3::new(-0.45, bed, -1.33),
            Vec3::new(0.25, bed + 0.06, -0.83),
            PANEL,
        );
        soup.slab(
            Vec3::new(0.6, bed, -1.3),
            Vec3::new(1.1, bed + 0.05, -0.9),
            PANEL,
        );
    });
    draft.light(
        Glow::Dock,
        Vec3::new(-0.5, 1.6, -1.38),
        Vec3::new(1.4, 1.64, -1.34),
    );
    draft.lamp(Glow::Dock, Vec3::new(0.45, 1.5, -0.9));
}

/// Adds the storage crate on four stubs, with a smaller case on top.
fn crates(draft: &mut Draft) {
    let (centre, half) = (BOX.0, BOX.1 * 0.5);
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .map(|(x, z)| centre + Vec2::new(x, z) * (half - 0.07));
    let bed = corners
        .iter()
        .map(|c| draft.ground(*c))
        .fold(f32::MIN, f32::max)
        + 0.06;
    for corner in corners {
        draft.leg(corner, bed + 0.02, 0.05, DARK);
    }
    let (lo, hi) = (centre - Vec2::splat(half), centre + Vec2::splat(half));
    let lid = bed + BOX.1;
    draft.deck(lo.extend(bed).xzy(), hi.extend(lid).xzy(), CRATE);
    draft.deck(
        Vec3::new(lo.x + 0.1, lid, lo.y + 0.15),
        Vec3::new(lo.x + 0.6, lid + 0.35, lo.y + 0.6),
        HATCH,
    );
    draft.block(
        lo.extend(draft.lowest(&corners)).xzy(),
        hi.extend(lid).xzy(),
    );
}

/// Adds the short mast with the beacon that marks home from afar.
fn mast(draft: &mut Draft) {
    let (top, half) = (HEIGHTS.2, Vec2::splat(0.13));
    draft.leg(MAST, top, 0.05, DARK);
    draft.draw(|soup| {
        soup.slab(
            (MAST - half * 0.7).extend(top - 0.3).xzy(),
            (MAST + half * 0.7).extend(top - 0.22).xzy(),
            TRIM,
        );
    });
    draft.light(
        Glow::Beacon,
        (MAST - half).extend(top).xzy(),
        (MAST + half).extend(top + 0.26).xzy(),
    );
    draft.lamp(Glow::Beacon, MAST.extend(top + 0.13).xzy());
    let low = draft.ground(MAST);
    draft.block(
        (MAST - half * 0.6).extend(low).xzy(),
        (MAST + half * 0.6).extend(top).xzy(),
    );
}

/// Draws the station at its spot.
pub fn build(draft: &mut Draft) {
    draft.stand(STATION, &POSTS, 0.0);
    shelter(draft);
    bench(draft);
    dock(draft);
    crates(draft);
    mast(draft);
}
