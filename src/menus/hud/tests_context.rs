//! Tests of the context rule: which situation the facts describe, and which elements each
//! situation shows.

use super::MarkerKind;
use super::context::{Context, Element, Facts, Set, context, marker_shown, shown};
use crate::ship::WarpState;
use crate::space::Frame;
use crate::universe::{BodyId, GalaxyId, StarId};

/// Facts of a player walking on the ground.
fn ground() -> Facts {
    Facts {
        control: true,
        seated: false,
        swimming: false,
        airborne: false,
        landed: true,
        frame: Frame::Surface(BodyId::default()),
        warp: WarpState::Off,
        bound: false,
    }
}

/// Facts of a player seated in a flying ship in `frame`.
fn flying(frame: Frame) -> Facts {
    Facts {
        seated: true,
        landed: false,
        frame,
        ..ground()
    }
}

/// Every situation is derived from the facts: on foot, swimming, gliding, each place of the
/// ship, warp and the menus.
#[test]
fn the_facts_give_the_context() {
    let star = Frame::System(StarId::default());
    let cases = [
        (ground(), Context::OnFoot),
        (Facts { swimming: true, ..ground() }, Context::Swimming),
        (Facts { airborne: true, ..ground() }, Context::Gliding),
        (Facts { control: false, ..ground() }, Context::Menu),
        (Facts { seated: true, ..ground() }, Context::ShipLanded),
        (flying(Frame::Surface(BodyId::default())), Context::ShipAir),
        (Facts { bound: true, ..flying(star) }, Context::ShipOrbit),
        (flying(star), Context::ShipSpace),
        (flying(Frame::Galaxy(GalaxyId::default())), Context::ShipBetween),
        (
            Facts { warp: WarpState::Spooling(0.3), ..flying(star) },
            Context::Warp,
        ),
        (
            Facts { warp: WarpState::Active(2.0e9), ..flying(star) },
            Context::Warp,
        ),
    ];
    for (facts, want) in cases {
        assert_eq!(context(&facts), want, "{facts:?}");
    }
}

/// Returns the elements a context shows, as a sorted list to compare.
fn of(context: Context) -> Vec<Element> {
    shown(context).elements()
}

/// What each context shows, listed exactly.
#[test]
fn each_context_shows_exactly_its_elements() {
    use Element::*;
    let table: [(Context, &[Element]); 10] = [
        (
            Context::OnFoot,
            &[Compass, Cardinals, Minimap, Coords, Place, Climate, Integrity, Prompt, Markers, Herds],
        ),
        (
            Context::Swimming,
            &[Compass, Cardinals, Minimap, Coords, Place, Climate, Integrity, Prompt, Markers],
        ),
        (
            Context::Gliding,
            &[
                Compass, Cardinals, Minimap, Coords, Place, Climate, Altitude, Speed, Integrity,
                Prompt, Markers, Herds,
            ],
        ),
        (
            Context::ShipLanded,
            &[
                Compass, Cardinals, Minimap, Coords, Place, Climate, Throttle, Assist, Gear,
                Integrity, Prompt, Markers,
            ],
        ),
        (
            Context::ShipAir,
            &[
                Compass, Cardinals, Minimap, Coords, Place, Climate, Altitude, Speed, Throttle,
                Assist, Gear, Integrity, Prompt, Markers, Herds,
            ],
        ),
        (
            Context::ShipOrbit,
            &[Compass, Altitude, Speed, Throttle, Assist, Apsides, Integrity, Prompt, Markers],
        ),
        (
            Context::ShipSpace,
            &[Compass, Speed, Throttle, Assist, Integrity, Prompt, Markers],
        ),
        (
            Context::ShipBetween,
            &[Compass, Speed, Throttle, Assist, Integrity, Prompt, Markers],
        ),
        (Context::Warp, &[Compass, Speed, Spool, Integrity, Prompt, Markers]),
        (Context::Menu, &[]),
    ];
    for (context, want) in table {
        assert_eq!(of(context), Set::of(want).elements(), "{context:?}");
    }
}

/// A reading whose data does not exist where the player is never shows: no altitude, orbit,
/// gear, north, ground coordinates, weather, minimap or herds in open space, and the orbit only
/// in an orbit.
#[test]
fn no_element_shows_where_its_data_is_missing() {
    let useless = [
        Element::Cardinals,
        Element::Minimap,
        Element::Coords,
        Element::Climate,
        Element::Gear,
        Element::Herds,
        Element::Place,
    ];
    for context in [Context::ShipOrbit, Context::ShipSpace, Context::ShipBetween, Context::Warp] {
        for element in useless {
            assert!(!shown(context).contains(element), "{context:?} {element:?}");
        }
    }
    for context in [Context::ShipSpace, Context::ShipBetween, Context::Warp] {
        assert!(!shown(context).contains(Element::Altitude), "{context:?}");
    }
    for element in [Element::Apsides, Element::Throttle, Element::Spool, Element::Assist] {
        for context in [Context::OnFoot, Context::Swimming, Context::Gliding, Context::Menu] {
            assert!(!shown(context).contains(element), "{context:?} {element:?}");
        }
    }
    for context in Context::ALL {
        let has = shown(context).contains(Element::Apsides);
        assert_eq!(has, context == Context::ShipOrbit, "{context:?}");
    }
}

/// In warp only the course marker shows, in space no pins, and herds only where herds show.
#[test]
fn markers_follow_the_context() {
    assert!(marker_shown(Context::Warp, MarkerKind::Course));
    assert!(!marker_shown(Context::Warp, MarkerKind::Home));
    assert!(!marker_shown(Context::ShipSpace, MarkerKind::Waypoint));
    assert!(marker_shown(Context::ShipSpace, MarkerKind::Course));
    assert!(marker_shown(Context::OnFoot, MarkerKind::Waypoint));
    assert!(marker_shown(Context::OnFoot, MarkerKind::Herd));
    assert!(!marker_shown(Context::Swimming, MarkerKind::Herd));
}
