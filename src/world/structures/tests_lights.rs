//! Tests that villages and camps carry the lights and fires they promise, and of the flames and light strengths.

use bevy::prelude::*;

use bevy::mesh::Indices;

use super::audit::{positions, signed_volume};
use super::brush::Build;
use super::camp::{Layout, draw_layout};
use super::flame::{emitter, pose, radiance, shells, teardrop};
use super::flicker::power;
use super::glow::{LightKind, WALL_REACH};
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

/// Torches and lamps give 150 to 400 lux at 3 m, every light reaches past 5 m, and a wall torch burns off its wall.
#[test]
fn lights_make_visible_pools() {
    let lux = |kind: LightKind| power(kind).0 / (4.0 * std::f32::consts::PI * 9.0);
    for kind in [LightKind::Torch, LightKind::Lamp] {
        assert!(
            (150.0..=400.0).contains(&lux(kind)),
            "{kind:?}: {} lux at 3 m",
            lux(kind)
        );
    }
    assert!(
        lux(LightKind::Campfire) > lux(LightKind::Brazier)
            && lux(LightKind::Candle) < lux(LightKind::Torch)
    );
    assert!(LightKind::ALL.iter().all(|kind| power(*kind).1 >= 6.0));
    let t = Terrain::new(1);
    let mut b = built(&t, Kind::Cabin, 3);
    let wall = Vec3::new(1.0, 1.6, 2.0);
    b.wall_torch(wall);
    let out = (b.lights[0].pos - b.pt(wall)).dot(b.dir(Vec3::Z));
    assert!(
        WALL_REACH >= 0.35 && (out - WALL_REACH).abs() < 1e-4,
        "torch is {out} m from its wall"
    );
}

/// The flame shell and the ray traced emitter are closed, wind outward and only the emitter is ray traced.
#[test]
fn flame_meshes_are_closed_and_fit_ray_tracing() {
    for (mesh, traced, tris) in [(teardrop(), false, 36), (emitter(), true, 8)] {
        let verts = positions(&mesh);
        assert_eq!(verts.len(), tris * 3);
        assert!(signed_volume(&verts) > 0.0, "mesh is inside out");
        assert_eq!(mesh.enable_raytracing, traced);
        assert!(matches!(mesh.indices(), Some(Indices::U32(_))));
        assert_eq!(mesh.attributes().count(), 4);
    }
    let (low, high) = positions(&teardrop())
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), p| {
            (lo.min(p.y), hi.max(p.y))
        });
    assert_eq!((low, high), (0.0, 1.0));
    let torch = radiance(LightKind::Torch);
    assert!(torch.red > 50_000.0 && torch.red > torch.green && torch.green > torch.blue);
}

/// Every kind has nested shells, big fires have three tongues, and a shell moves without its base leaving its spot.
#[test]
fn flame_shells_nest_and_pivot_on_their_base() {
    for kind in LightKind::ALL {
        let flame = shells(kind, 1.3);
        let big = matches!(kind, LightKind::Campfire | LightKind::Hearth);
        assert!(flame.len() >= 3 && flame.len().is_multiple_of(3) && (!big || flame.len() == 9));
        for tongue in flame.chunks(3) {
            assert_eq!([tongue[0].0, tongue[1].0, tongue[2].0], [0, 1, 2]);
            assert!(
                tongue
                    .windows(2)
                    .all(|w| w[1].1.size.cmplt(w[0].1.size).all())
            );
        }
        for (frame, (_, layer)) in (0..300).flat_map(|f| flame.iter().map(move |l| (f, l))) {
            let at = pose(layer, frame as f32 * 0.05, Vec3::new(4.0, 0.0, -3.0));
            let stretch = at.scale.y / layer.size.y;
            assert_eq!(at.translation, layer.base);
            assert!(
                (0.7..=1.3).contains(&stretch) && at.rotation.is_normalized(),
                "{kind:?}: {stretch}"
            );
        }
    }
    let lean = |seconds: f32| pose(&shells(LightKind::Torch, 0.0)[0].1, seconds, Vec3::ZERO);
    assert!(lean(0.3).scale.y != lean(0.9).scale.y && lean(0.3).rotation != lean(0.9).rotation);
}
