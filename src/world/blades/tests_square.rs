//! Test of the landing pad where blades certainly grow: a lush planet, whose pad bares its square and nothing else.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Cropped;
use super::clear::blocked;
use super::ground::Ground;
use super::tests_tile::{turfed, view};
use super::tile::grow;
use super::tune::PAD_MARGIN;
use crate::world::Globe;
use crate::world::Terrain;
use crate::world::globe::{Pad, Reserve};
use crate::world::scatter::Carpet;
use crate::world::scatter::zones::hold;

/// Returns the planet position of the root of every blade of the dense tile `key` of `terrain` over `ground`
/// and `carpet`, but those `clear` says are taken.
fn roots(
    (terrain, ground, key, carpet): (&Terrain, &Ground, IVec2, &Carpet),
    clear: &dyn Fn(DVec3) -> bool,
) -> Vec<DVec3> {
    let (blades, frame) = grow((terrain, ground), key, carpet, true, clear);
    let at = |root: &[f32; 3]| frame.origin + frame.rot * Vec3::from_array(*root).as_dvec3();
    blades.roots().iter().step_by(3).map(at).collect()
}

/// Returns `globe` with a pad of two metres a side kept bare around planet position `at`.
fn padded(globe: &Arc<Globe>, at: DVec3) -> Arc<Globe> {
    let up = at.normalize();
    let heading = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let square = Pad {
        centre: at,
        up,
        heading,
        half: 1.0,
    };
    let reserve = Reserve {
        centre: at,
        up,
        radius: 0.0,
        pad: Some(square),
    };
    Arc::new(globe.as_ref().clone().reserving(reserve))
}

/// Where blades certainly grow, a pad of two metres a side bares its square and a margin of one blade and nothing
/// else: the same tile grows blades there without the pad, none with it, and the same ones everywhere else.
#[test]
fn the_pad_bares_only_its_square() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, _) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let (_, frame) = grow((&terrain, &ground), key, &carpet, true, &|_| false);
    let shut = padded(terrain.globe(), frame.origin);
    let none = Cropped::default();
    let tile = (&terrain, &ground, key, &carpet);
    let all = roots(tile, &|_| false);
    let kept = roots(tile, &|p| blocked(p, (&[], &none), &shut));
    let under = all.iter().filter(|p| shut.on_pad(**p, PAD_MARGIN)).count();
    println!(
        "PAD tile blades {}, under the pad {under}, kept {}",
        all.len(),
        kept.len()
    );
    assert!(under > 0, "the tile grows no blade where the pad is put");
    assert!(kept.iter().all(|p| !shut.on_pad(*p, PAD_MARGIN)));
    assert_eq!(
        kept.len(),
        all.len() - under,
        "blades were lost outside the pad"
    );
}
