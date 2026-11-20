//! Tests of the elements each context shows, listed exactly.

use super::context::{Context, Element, Set, shown};

/// Asserts that `context` shows exactly `want`.
fn exactly(context: Context, want: &[Element]) {
    assert_eq!(
        shown(context).elements(),
        Set::of(want).elements(),
        "{context:?}"
    );
}

/// On foot, swimming and gliding show the ground elements, and gliding adds its two rows.
#[test]
fn the_body_shows_the_ground_elements() {
    use Element::*;
    let ground = [
        Compass, Cardinals, Minimap, Coords, Place, Climate, Integrity, Prompt, Markers,
    ];
    exactly(Context::OnFoot, &[&ground[..], &[Herds]].concat());
    exactly(Context::Swimming, &ground);
    exactly(
        Context::Gliding,
        &[&ground[..], &[Altitude, Speed, Herds]].concat(),
    );
}

/// A ship near the ground shows the ground elements and its rows.
#[test]
fn the_ship_near_the_ground_shows_its_rows() {
    use Element::*;
    let ground = [
        Compass, Cardinals, Minimap, Coords, Place, Climate, Integrity, Prompt, Markers,
    ];
    let helm = [Throttle, Assist, Gear];
    exactly(Context::ShipLanded, &[&ground[..], &helm[..]].concat());
    let air = [Altitude, Speed, Herds];
    exactly(
        Context::ShipAir,
        &[&ground[..], &helm[..], &air[..]].concat(),
    );
}

/// In space the ship shows no ground element, and in warp almost nothing.
#[test]
fn the_ship_in_space_shows_its_engine_and_course() {
    use Element::*;
    let base = [Compass, Integrity, Prompt, Markers];
    exactly(
        Context::ShipOrbit,
        &[&base[..], &[Altitude, Speed, Throttle, Assist, Apsides]].concat(),
    );
    exactly(
        Context::ShipSpace,
        &[&base[..], &[Speed, Throttle, Assist]].concat(),
    );
    exactly(
        Context::ShipBetween,
        &[&base[..], &[Speed, Throttle, Assist]].concat(),
    );
    exactly(Context::Warp, &[&base[..], &[Speed, Spool]].concat());
    exactly(Context::Menu, &[]);
}

/// No ground element and no gear shows in space, and no altitude shows in open space.
#[test]
fn no_ground_element_shows_in_space() {
    let useless = [
        Element::Cardinals,
        Element::Minimap,
        Element::Coords,
        Element::Climate,
    ];
    let more = [Element::Gear, Element::Herds, Element::Place];
    for context in [
        Context::ShipOrbit,
        Context::ShipSpace,
        Context::ShipBetween,
        Context::Warp,
    ] {
        for element in useless.into_iter().chain(more) {
            assert!(!shown(context).contains(element), "{context:?} {element:?}");
        }
    }
    for context in [Context::ShipSpace, Context::ShipBetween, Context::Warp] {
        assert!(!shown(context).contains(Element::Altitude), "{context:?}");
    }
}

/// The orbit shows only in an orbit, and the engine and the warp spool only in the ship.
#[test]
fn engine_and_orbit_readings_need_the_ship() {
    for element in [
        Element::Apsides,
        Element::Throttle,
        Element::Spool,
        Element::Assist,
    ] {
        for context in [
            Context::OnFoot,
            Context::Swimming,
            Context::Gliding,
            Context::Menu,
        ] {
            assert!(!shown(context).contains(element), "{context:?} {element:?}");
        }
    }
    for context in Context::ALL {
        let has = shown(context).contains(Element::Apsides);
        assert_eq!(has, context == Context::ShipOrbit, "{context:?}");
    }
}
