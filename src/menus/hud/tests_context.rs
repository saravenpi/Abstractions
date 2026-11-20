//! Tests of the context rule: which situation the facts describe, and which markers show there.

use super::MarkerKind;
use super::context::{Context, Facts, context, marker_shown};
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

/// Checks that each of `cases` gives its context.
fn check(cases: &[(Facts, Context)]) {
    for (facts, want) in cases {
        assert_eq!(context(facts), *want, "{facts:?}");
    }
}

/// On foot, swimming, gliding and in a menu come from the body and the control.
#[test]
fn the_body_gives_the_context_on_foot() {
    check(&[
        (ground(), Context::OnFoot),
        (
            Facts {
                swimming: true,
                ..ground()
            },
            Context::Swimming,
        ),
        (
            Facts {
                airborne: true,
                ..ground()
            },
            Context::Gliding,
        ),
        (
            Facts {
                control: false,
                ..ground()
            },
            Context::Menu,
        ),
    ]);
}

/// Seated, the frame, the ground and the orbit give the place of the ship.
#[test]
fn the_ship_gives_the_context_seated() {
    let star = Frame::System(StarId::default());
    check(&[
        (
            Facts {
                seated: true,
                ..ground()
            },
            Context::ShipLanded,
        ),
        (flying(Frame::Surface(BodyId::default())), Context::ShipAir),
        (
            Facts {
                bound: true,
                ..flying(star)
            },
            Context::ShipOrbit,
        ),
        (flying(star), Context::ShipSpace),
        (
            flying(Frame::Galaxy(GalaxyId::default())),
            Context::ShipBetween,
        ),
    ]);
}

/// A spooling or running warp drive is warp, wherever the ship is.
#[test]
fn the_warp_drive_gives_warp() {
    let star = Frame::System(StarId::default());
    for warp in [WarpState::Spooling(0.3), WarpState::Active(2.0e9)] {
        check(&[(
            Facts {
                warp,
                ..flying(star)
            },
            Context::Warp,
        )]);
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
