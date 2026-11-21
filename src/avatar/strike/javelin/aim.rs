//! Aim: where a line meets the drawn ground, the point a javelin is thrown at where the crosshair
//! ray first meets the ground or an animal, and the throw solved from the hand so that it leaves
//! straight for that point though the hand is not where the eyes are.

use bevy::prelude::*;

use super::ground::Ground;
use super::tune::{HALVINGS, MARCH, THROW_SIGHT};

/// Returns the point a javelin is thrown at along the crosshair ray from `eye` along the unit `dir`:
/// the animal `metres` along it when one is there, else where the ray first meets `ground` within
/// [`THROW_SIGHT`], else the ray at that distance.
pub fn far(ground: Option<&impl Ground>, (eye, dir): (Vec3, Vec3), metres: Option<f32>) -> Vec3 {
    let most = metres.unwrap_or(THROW_SIGHT).min(THROW_SIGHT);
    let end = eye + dir * most;
    let Some(ground) = ground else {
        return end;
    };
    let (first, grow) = MARCH;
    let mut at = 0.0;
    while at < most {
        let next = (at + first + grow * at).min(most);
        if let Some(met) = cross(ground, eye + dir * at, eye + dir * next) {
            return met;
        }
        at = next;
    }
    end
}

/// Returns where a javelin of `reach` metres from the hand to the tip held at `grip` starts and the
/// unit way it leaves along to pass through `target`: its tip on the line from the hand to the
/// target, so the eyes and the hand see the same throw.
pub fn launch(grip: Vec3, target: Vec3, reach: f32) -> (Vec3, Vec3) {
    let way = (target - grip).normalize_or(Vec3::NEG_Z);
    (grip + way * reach, way)
}

/// Returns where the step from `from` to `to` meets `ground`, when it ends under it: found by
/// halving the step, always on or just above the ground. A step that starts under the ground meets
/// it straight over where it starts.
pub fn cross(ground: &impl Ground, from: Vec3, to: Vec3) -> Option<Vec3> {
    if ground.clearance(to) >= 0.0 {
        return None;
    }
    if ground.clearance(from) < 0.0 {
        return Some(ground.lift(from));
    }
    let (mut above, mut under) = (from, to);
    for _ in 0..HALVINGS {
        let mid = above.lerp(under, 0.5);
        match ground.clearance(mid) >= 0.0 {
            true => above = mid,
            false => under = mid,
        }
    }
    Some(above)
}
