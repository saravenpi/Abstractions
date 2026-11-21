//! The ship casts its shadow: the plates, the trim, the engines and the gear of the hull are drawn
//! into the shadow maps, for the ship the player flies and for a puppet alike.

use bevy::camera::visibility::VisibilityRange;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::stage::{Puppet, app};
use crate::ship::body::Ship;
use crate::ship::mesh::{FAR, Looks, Skin};

/// The skins of the outside of the hull, which must cast the shadow of the ship.
const OUTSIDE: [Skin; 3] = [Skin::Hull, Skin::Trim, Skin::Dark];

/// Returns how many outside parts of the detailed hull of the ship marked `S` cast a shadow and
/// how many do not. The parts of the cockpit, which cast none on purpose, are left out by their
/// own shorter range.
fn casting<S: Component>(app: &mut App) -> (usize, usize) {
    let skins = app.world().resource::<Looks>().skins.clone();
    let outside = OUTSIDE.map(|skin| skins[skin as usize].id());
    let mut ships = app.world_mut().query_filtered::<Entity, With<S>>();
    let ship = ships.single(app.world()).expect("one such ship");
    let mut parts = app.world_mut().query::<(
        &ChildOf,
        &MeshMaterial3d<StandardMaterial>,
        &VisibilityRange,
        Has<NotShadowCaster>,
    )>();
    let mut counts = (0, 0);
    for (parent, material, range, hidden) in parts.iter(app.world()) {
        let hull = parent.parent() == ship && outside.contains(&material.0.id());
        if hull && range.end_margin.start == FAR {
            match hidden {
                true => counts.1 += 1,
                false => counts.0 += 1,
            }
        }
    }
    counts
}

/// Both ships cast the shadow of their hull from every outside part, and the outside skins are
/// opaque, which the shadow pass draws.
#[test]
fn the_hull_casts_its_shadow() {
    let mut app = app();
    app.update();
    for counts in [casting::<Ship>(&mut app), casting::<Puppet>(&mut app)] {
        assert!(counts.0 >= 3 && counts.1 == 0, "{counts:?}");
    }
    let skins = app.world().resource::<Looks>().skins.clone();
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    for skin in OUTSIDE {
        let alpha = materials
            .get(&skins[skin as usize])
            .map(|own| own.alpha_mode);
        assert_eq!(alpha, Some(AlphaMode::Opaque), "{skin:?}");
    }
}
