//! Tests of what the game reads from the body: its triangle budgets, the eye on the face, the
//! screen in its visor, the grip in the hand and the soles on the ground.

use bevy::prelude::*;

use super::{EXTRA_TRIANGLES, SCREEN_FRONT, build, fabric, glow, screen};
use crate::avatar::rig::Seg;
use crate::avatar::tune::{ANKLE_RISE, BOOT, HEAD, SCREEN};
use crate::skin::Detail;
use crate::skin::rig::{inside, pieces};
use crate::view::FACE;

/// Most triangles the fifteen segments have up close, and from afar: the 808 of the body before
/// it had two levels.
const BUDGET: (usize, usize) = (3500, 808);

/// Returns the number of triangles of the fifteen segments at the level `detail`.
fn count(detail: Detail) -> usize {
    Seg::ALL.into_iter().map(|seg| build(seg, detail).triangles()).sum()
}

/// The body stays inside its budget at both levels, the far level is no more than the old body,
/// and the parts that are not segments count what the budget says.
#[test]
fn triangle_budgets_hold() {
    let (near, far) = (count(Detail::Near), count(Detail::Far));
    assert!(near <= BUDGET.0 && near > 2000, "{near} triangles up close");
    assert!(far <= BUDGET.1 && far > 400, "{far} triangles from afar");
    let extras = [screen(), glow(0.0), glow(0.0), glow(1.0), glow(1.0)];
    let cloth = [fabric(), fabric(), fabric()];
    let counted = extras.iter().chain(&cloth).map(|mesh| mesh.count_vertices() / 3);
    assert_eq!(counted.sum::<usize>(), EXTRA_TRIANGLES);
    for seg in Seg::ALL {
        assert!(build(seg, Detail::Near).points().all(|p| p.is_finite()));
    }
}

/// The eye of the first person view is on the screen: inside its rectangle and within a
/// centimetre of its plane. The screen lies on the front of the visor box, a hair proud of it.
#[test]
fn the_eye_is_on_the_face() {
    assert!(FACE.x.abs() < SCREEN[0] && (FACE.y - HEAD.0).abs() < SCREEN[1]);
    assert!((FACE.z + SCREEN_FRONT).abs() < 0.01, "the eye is {} deep", FACE.z);
    let head = pieces(&build(Seg::Head, Detail::Near));
    for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (0.0, 0.0)] {
        let on = Vec3::new(x * SCREEN[0], HEAD.0 + y * SCREEN[1], -SCREEN_FRONT);
        let behind = on + Vec3::Z * 0.004;
        let held = |p: Vec3| head.iter().any(|shape| inside(shape, p));
        assert!(held(behind) && !held(on), "the screen floats or is buried at {on}");
    }
}

/// The grip of a held tool, 45 mm under the wrist, is inside the hand, and the hand ends within
/// eleven centimetres of the wrist, where the ground and the controls expect the palm.
#[test]
fn the_grip_is_in_the_hand() {
    for seg in [Seg::HandL, Seg::HandR] {
        let hand = build(seg, Detail::Near);
        let shapes = pieces(&hand);
        assert!(shapes.iter().any(|shape| inside(shape, Vec3::NEG_Y * 0.045)));
        let low = hand.points().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!(low > -0.11 && low < -0.08, "the hand ends at {low}");
    }
}

/// The sole of each boot is flat at the height of the ankle above the ground, as long as the
/// box of the boot, and nothing of the boot is under it.
#[test]
fn the_soles_are_flat_on_the_ground() {
    for seg in [Seg::BootL, Seg::BootR] {
        let boot = build(seg, Detail::Near);
        let low = boot.points().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((low + ANKLE_RISE).abs() < 1e-5, "the sole is at {low}");
        let under: Vec<Vec3> = boot.points().filter(|p| p.y < low + 1e-4).collect();
        let reach = |pick: fn(f32, f32) -> f32, start: f32| under.iter().map(|p| p.z).fold(start, pick);
        let (toe, heel) = (reach(f32::min, f32::MAX), reach(f32::max, f32::MIN));
        assert!(toe < BOOT.0[2] - BOOT.1[2] + 0.02 && heel > BOOT.0[2] + BOOT.1[2] - 0.02);
    }
}
