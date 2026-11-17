//! Tests of the survey of sites and built floors around a point.

use bevy::prelude::*;

use super::globes::{RADII, faces, villages};
use super::site::{Kind, Site, structure_in_cell};
use super::{FloorKind, StructureData, Survey, Terrain, survey, village};

/// Asserts that `survey` knows the village `site` of the chart `face`: it is the nearest site from its
/// plaza, the middle of every house floor is wood, the plaza is a built floor and the sky above is not.
fn check_survey(face: &Terrain, site: Site) {
    let view = face.around(site.origin.x, site.origin.y);
    let at = |p: Vec2, y: f32| view.embed(p.x, y, p.y);
    let plaza = at(site.origin, site.floor);
    let seen: Survey = survey(face, plaza, 120.0);
    let near = seen.nearest(plaza, 100.0).expect("the village is in reach");
    assert!(near.kind == Kind::Village && near.distance < 0.5 && near.reach >= 60.0);
    assert!(
        seen.nearest(
            plaza + (plaza - at(site.origin, 0.0)).normalize() * 500.0,
            100.0
        )
        .is_none()
    );
    for lot in village::plots(&view, site) {
        let floor = seen.floor_at(at(lot.pos, lot.floor + 0.02));
        let stall = lot.house == village::House::Stall;
        assert!(
            stall || floor == Some(FloorKind::Wood),
            "{lot:?} has floor {floor:?}"
        );
        assert!(floor.is_some(), "{lot:?} has no built floor");
    }
    assert!(
        seen.floor_at(plaza).is_some(),
        "the plaza is not a built floor"
    );
    assert!(seen.floor_at(at(site.origin, site.floor + 30.0)).is_none());
}

/// The survey finds villages and their floors on flat land and on globes of 3 and 40 km, and what the
/// chunk workers and the survey produce can be sent between threads.
#[test]
fn surveys_find_sites_and_floors() {
    fn sendable<T: Send>() {}
    sendable::<StructureData>();
    sendable::<Survey>();
    let flat = Terrain::new(1);
    let cells = (-12..12).flat_map(|x| (-12..12).map(move |y| IVec2::new(x, y)));
    let mut found: Vec<(Terrain, Site)> = cells
        .filter_map(|c| structure_in_cell(&flat, c))
        .filter(|s| s.kind == Kind::Village)
        .map(|s| (flat.clone(), s))
        .collect();
    for radius in RADII {
        for face in faces(42, radius).iter().take(3) {
            found.extend(villages(face, 1).into_iter().map(|s| (face.clone(), s)));
        }
    }
    assert!(found.len() >= 5, "{} villages", found.len());
    for (face, site) in found {
        check_survey(&face, site);
    }
}
