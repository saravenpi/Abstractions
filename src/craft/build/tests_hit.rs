//! Tests of blows against layers and of thrown spears.

use super::rig::{earth, example};
use super::*;
use crate::craft::matter::{Class, Mineral};

/// The worked example with a flint edge slashes through the shell and the hide under it, and keeps
/// less energy than it started with minus the work of both layers.
#[test]
fn a_hard_edge_cuts_through() {
    let build = example(Form::Edge, Mineral::Flint.material());
    let hit = blow(&build, Intent::Cut, &earth());
    let done = strike(&hit, &reference());
    assert_eq!((done.passed, done.soft), (2, None), "{done:?}");
    assert!(done.left > 0.0 && done.left < hit.energy * KEPT2);
}

/// Share of energy two layers leave at most: 5 % of momentum lost per layer.
const KEPT2: f32 = 0.95 * 0.95 * 0.95 * 0.95;

/// A hide edge is no harder than shell: it turns blunt on the first layer and says so.
#[test]
fn a_soft_edge_turns_blunt() {
    let build = example(Form::Edge, Class::Hide.anchor());
    let done = strike(&blow(&build, Intent::Cut, &earth()), &reference());
    assert_eq!(done.soft, Some(0));
    let flint = strike(
        &blow(
            &example(Form::Edge, Mineral::Flint.material()),
            Intent::Cut,
            &earth(),
        ),
        &reference(),
    );
    assert!(done.score() < flint.score());
}

/// A crushing head breaks shell with enough energy and stops on a thick layer it cannot break.
#[test]
fn blunt_blows_break_or_stop() {
    let layers = reference();
    let blow = |energy: f32| Blow {
        energy,
        mode: Mode::Blunt,
        width: 0.06,
        area: 0.0036,
        hardness: 6.0,
    };
    assert!(strike(&blow(200.0), &layers[..1]).passed == 1);
    let weak = strike(&blow(0.5), &layers[..1]);
    assert!(weak.passed == 0 && weak.part > 0.0 && weak.part < 1.0);
}

/// A spear with its mass forward flies point first and slows with distance; one weighted at the butt
/// tumbles; thinner air drags less.
#[test]
fn spears_fly_point_first() {
    let spear = Build {
        shaft: Shaft {
            length: 1.2,
            ..example(Form::Point, Mineral::Flint.material()).shaft
        },
        head: Head {
            mass: 0.2,
            ..example(Form::Point, Mineral::Flint.material()).head
        },
        ..example(Form::Point, Mineral::Flint.material())
    };
    let flight = throw(&spear, &earth());
    assert!(flight.stable && flight.speed_at(20.0) < flight.launch);
    let tail = Build {
        grip: 0.15,
        counterweight: Some(Weight {
            mass: 1.5,
            material: Mineral::Basalt.material(),
        }),
        ..spear
    };
    assert!(!throw(&tail, &earth()).stable);
    let thin = Place {
        air: 0.1,
        ..earth()
    };
    assert!(throw(&spear, &thin).drag < flight.drag);
}
