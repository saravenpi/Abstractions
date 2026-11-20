//! The rosette: a ring of blades round a heart, on the ground or on the short trunk of an old plant.

use bevy::prelude::*;

use super::kit::{Kit, OLD};
use super::low::{deeper, ring};
use crate::world::solids::{fan, icosa};

/// Returns the heart of the rosette of `k`: on the ground, or atop a short leaning trunk it builds when the
/// plant is old and of a branching species.
fn heart(k: &mut Kit) -> Vec3 {
    if k.age < OLD || k.dials.kind.branches < 3 {
        return Vec3::Y * 0.02;
    }
    let top = k.axis() * 0.6 + Vec3::Y * 0.1;
    k.stem(Vec3::new(0.0, -0.3, 0.0), top, [0.09, 0.07]);
    top
}

/// Adds two small side rosettes of blades `width` wide round a rosette of `reach` and `rise` turned by `yaw`,
/// when its species clumps and it is past its youth.
fn pups(k: &mut Kit, (reach, rise, width): (f32, f32, f32), yaw: f32, colour: [f32; 4]) {
    if k.dials.kind.branches < 4 || k.age == 0 {
        return;
    }
    for s in 0..2 {
        let a = yaw + 1.3 + s as f32 * 2.9;
        let pup = [5.0, reach * 0.45, rise * 0.4, 0.0, width * 0.7];
        fan(
            &mut k.mesh,
            Vec3::new(a.cos(), 0.0, a.sin()) * reach * 0.9,
            pup,
            a,
            colour,
        );
    }
}

/// Adds the flower spike of a rosette over `heart` when it flowers or fruits, dressed along its top.
fn spire(k: &mut Kit, heart: Vec3) {
    if k.dials.paint.flowers.is_none() && k.dials.paint.fruit.is_none() {
        return;
    }
    let head = heart + Vec3::new(0.05 * k.rand(2), 1.3 * k.grow, 0.05 * k.rand(3));
    k.stem(heart, head, [0.04, 0.03]);
    k.dress(&[head, head.lerp(heart, 0.15), head.lerp(heart, 0.3)], 0.08);
}

/// Returns the outer ring of blades of a rosette: as many, long, upright and arched as its species, fewer
/// when young and one more when old.
fn blades(k: &Kit) -> [f32; 5] {
    let (form, kind, grow) = (k.dials.form, k.dials.kind, k.grow);
    let count = 4 + u32::from(kind.leaves) + u32::from(k.age == OLD) - 2 * u32::from(k.age == 0);
    let reach = (0.4 + 0.25 * kind.spread) * grow;
    let rise = (0.75 - 0.35 * kind.droop - 0.15 * kind.spread) * grow;
    let width = (0.06 + 0.18 * form.leaf.min(1.4)) * grow;
    [
        count.min(12) as f32,
        reach,
        rise,
        0.1 + 0.25 * kind.droop,
        width,
    ]
}

/// Builds a rosette: a ring of blades, a second ring under wax, a swollen heart where it stores water, spines
/// in drought, side shoots when it clumps, a flower spike when it flowers, and a short leaning trunk under the
/// rosette of an old one.
pub fn rosette(k: &mut Kit) {
    let (form, colour, yaw) = (k.dials.form, k.leaves(), k.rand(1) * 3.0);
    let heart = heart(k);
    let outer = blades(k);
    let [count, reach, rise, _, width] = outer;
    fan(&mut k.mesh, heart, outer, yaw, colour);
    if form.wax {
        let inner = [count - 3.0, reach * 0.6, rise * 0.9, 0.08, width * 0.8];
        fan(
            &mut k.mesh,
            heart + Vec3::Y * 0.02,
            inner,
            yaw + 0.3,
            deeper(colour),
        );
    }
    if form.store {
        let swollen = Vec3::new(0.25, 0.2, 0.25) * k.grow;
        icosa(
            &mut k.mesh,
            heart + Vec3::Y * 0.13,
            swollen,
            (0.15, 5),
            colour,
        );
    }
    if form.spines {
        ring(k, heart, 9, (0.75, reach * 0.9, yaw), 0.12);
    }
    pups(k, (reach, rise, width), yaw, colour);
    spire(k, heart);
}
