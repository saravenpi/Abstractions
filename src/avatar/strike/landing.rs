//! Landing: what a thrown spear carries when it arrives, as a blow on an animal and as an impact on
//! the ground.

use bevy::prelude::*;

use super::flight::{Rest, Spear};
use super::hit::{Hit, Tip};
use super::tune::GROUND;
use crate::audio::{Body, Impact, Outer};
use crate::craft::build::{Intent, blow};

/// Returns the hit `spear` lands now on `part` of `victim` at `at`, carrying its kinetic energy
/// ½mv².
pub fn arrival(spear: &Spear, (victim, part, at): (Entity, Entity, Vec3)) -> Hit {
    let speed = spear.velocity.length();
    let mut landed = blow(&spear.item.build, Intent::Throw, &spear.item.place);
    landed.energy = 0.5 * spear.item.throw.mass * speed * speed;
    Hit {
        attacker: spear.body,
        item: spear.item.id,
        victim,
        part,
        at,
        along: spear.velocity / speed.max(f32::EPSILON),
        blow: landed,
        intent: Intent::Throw,
        tip: Tip {
            stiffness: spear.item.build.head.material.stiffness,
            mass: spear.item.build.head.mass,
        },
    }
}

/// Returns true when `spear` is in the air and its point moved to `now` by no more than a frame of
/// flight could take it: a rebase of the world in between is not a path through it.
pub fn steady(spear: &Spear, now: Vec3) -> bool {
    spear.rest == Rest::Flying && now.distance(spear.was) <= spear.velocity.length() * 0.2 + 1.0
}

/// Returns the impact of `spear` arriving at `point` in the ground: its energy on soil, which gives
/// way far more than any animal.
pub fn grounded(spear: &Spear, point: Vec3) -> Impact {
    let speed = spear.velocity.length();
    let head = &spear.item.build.head;
    Impact {
        at: point,
        joules: 0.5 * spear.item.throw.mass * speed * speed,
        cut: false,
        outer: Outer::Ground,
        struck: Body {
            stiffness: GROUND.0,
            mass: GROUND.1,
        },
        tool: Body {
            stiffness: head.material.stiffness,
            mass: head.mass,
        },
        deflected: false,
    }
}
