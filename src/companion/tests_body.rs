//! Tests of the companion's meshes: closed shapes that face outward, one piece, inside the
//! budget at both levels, and what the game reads from them.

use bevy::prelude::*;

use super::body::{ANTENNA_TIP, NOZZLE, SCREEN, SCREEN_FRONT, mitt, shell};
use super::tune::{HALF, SHELL_RADIUS};
use crate::skin::rig::{floating, inside, open_edges, pieces, volume};
use crate::skin::{Detail, Soup};

/// Most triangles the head-body and the two hands have up close, and from afar: the 224 of the
/// companion before it had two levels.
const BUDGET: (usize, usize) = (1000, 224);

/// Returns the head-body and a hand at the level `detail`.
fn parts(detail: Detail) -> [Soup; 2] {
    [shell(detail), mitt(detail)]
}

/// Every shape is closed and faces outward at both levels, and no triangle is degenerate.
#[test]
fn every_shape_is_closed_and_faces_outward() {
    for (detail, soup) in [Detail::Near, Detail::Far]
        .into_iter()
        .flat_map(|d| parts(d).map(|s| (d, s)))
    {
        for tri in soup.tris() {
            let area = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length();
            assert!(area > 1e-8, "{detail:?}: degenerate {tri:?}");
        }
        for piece in pieces(&soup) {
            assert_eq!(open_edges(&piece), 0, "{detail:?}: open at {:?}", piece[0]);
            assert!(
                volume(&piece) > 1e-7,
                "{detail:?}: inside out at {:?}",
                piece[0]
            );
        }
    }
}

/// The shapes of the head-body are one piece at both levels, and a hand is one shape.
#[test]
fn no_shape_floats() {
    for detail in [Detail::Near, Detail::Far] {
        let all = pieces(&shell(detail));
        let lost: Vec<Vec3> = floating(&all).into_iter().map(|i| all[i][0][0]).collect();
        assert!(lost.is_empty(), "{detail:?}: shapes float near {lost:?}");
        assert_eq!(pieces(&mitt(detail)).len(), 1);
    }
}

/// The companion stays inside its budget at both levels.
#[test]
fn triangle_budgets_hold() {
    let count = |detail| shell(detail).triangles() + 2 * mitt(detail).triangles();
    let (near, far) = (count(Detail::Near), count(Detail::Far));
    assert!(near <= BUDGET.0 && near > 500, "{near} triangles up close");
    assert!(far <= BUDGET.1 && far > 150, "{far} triangles from afar");
}

/// The screen lies on the front of the visor box, a hair proud of it; the antenna tip sits on
/// the top of the crest and the thruster glows in the mouth of the bell; and the body stays
/// inside the ball it is to the solids, but for the crest over its crown.
#[test]
fn the_glowing_parts_sit_on_the_body() {
    let body = pieces(&shell(Detail::Near));
    let held = |p: Vec3| body.iter().any(|shape| inside(shape, p));
    for (x, y) in [
        (-1.0, -1.0),
        (1.0, -1.0),
        (1.0, 1.0),
        (-1.0, 1.0),
        (0.0, 0.0),
    ] {
        let on = Vec3::new(x * SCREEN.x, y * SCREEN.y, -SCREEN_FRONT);
        assert!(
            held(on + Vec3::Z * 0.004) && !held(on),
            "the screen is off the visor at {on}"
        );
    }
    assert!(
        held(ANTENNA_TIP + Vec3::new(0.0, -0.015, -0.003)),
        "the tip is off the crest"
    );
    assert!(held(NOZZLE + Vec3::Y * 0.02) && !held(NOZZLE - Vec3::Y * 0.03));
    let whole = shell(Detail::Near);
    let under = whole.points().filter(|p| p.y < HALF[1]);
    let far = under.map(|p| p.length()).fold(0.0, f32::max);
    assert!(
        far < SHELL_RADIUS + 0.01,
        "the body reaches {far} m from its middle"
    );
}
