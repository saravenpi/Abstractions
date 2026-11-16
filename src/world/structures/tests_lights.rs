//! Tests that villages and camps carry the lights and fires they promise.

use bevy::prelude::*;

use super::audit::{positions, signed_volume};
use super::brush::Build;
use super::camp::{Layout, draw_layout};
use super::glow::LightKind;
use super::site::{CELL, Kind, structure_in_cell};
use super::tests::built;
use super::{DRAW, Terrain};

/// Counts the lights of `kind` in a build.
fn count(b: &Build, kind: LightKind) -> usize {
    b.lights.iter().filter(|l| l.kind == kind).count()
}

/// Every camp layout except the abandoned one burns a campfire, and the abandoned one burns none.
#[test]
fn camps_burn_fires() {
    let t = Terrain::new(1);
    for layout in Layout::ALL {
        let mut b = built(&t, Kind::Camp, 3);
        draw_layout(&mut b, layout);
        let fires = count(&b, LightKind::Campfire);
        assert_eq!(
            fires == 0,
            layout == Layout::Abandoned,
            "{layout:?} has {fires} fires"
        );
    }
}

/// Every village has lamps, door torches and a brazier; the largest is reported.
#[test]
fn villages_are_lit() {
    let n = (2000.0 / CELL) as i32 + 1;
    let mut most = 0;
    for seed in 1..=3 {
        let t = Terrain::new(seed);
        let sites = (-n..n)
            .flat_map(|x| (-n..n).map(move |y| IVec2::new(x, y)))
            .filter_map(|c| structure_in_cell(&t, c))
            .filter(|s| s.kind == Kind::Village);
        for site in sites {
            let mut b = Build::new(&t, site.origin, site);
            DRAW[Kind::Village as usize](&mut b);
            assert!(count(&b, LightKind::Lamp) >= 4 && count(&b, LightKind::Torch) >= 6);
            assert_eq!(count(&b, LightKind::Brazier), 1);
            most = most.max(b.lights.len());
        }
    }
    eprintln!("most lights in one village: {most}");
}

/// Cones and prisms are closed and wind outward.
#[test]
fn round_solids_wind_outward() {
    let t = Terrain::new(1);
    let mut b = built(&t, Kind::Cabin, 3);
    b.spire(Vec3::ZERO, Vec2::new(1.0, 2.0), 6, [0.5; 3]);
    assert!(
        signed_volume(&positions(&b.mesh.clone().into_mesh())) > 0.0,
        "cone is inside out"
    );
    let mut b = built(&t, Kind::Cabin, 3);
    b.pillar(
        Vec3::ZERO,
        Vec2::new(1.0, 2.0),
        7,
        super::brush::Mat::deco([0.5; 3]),
    );
    assert!(
        signed_volume(&positions(&b.mesh.clone().into_mesh())) > 0.0,
        "prism is inside out"
    );
}
