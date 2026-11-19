//! The layers that ride on the root: the head's look and the chains that follow the body.

use bevy::math::{Quat, Vec2, Vec3};

use super::frame::{Kind, Role, chains};
use super::ik::unit;
use super::look::aim;
use super::rest::carry;
use super::rig::{Detail, Motion};
use super::spring::Spring;
use super::strand::Pull;
use crate::life::body::{Body, Locomotion};

/// Turns the head of `m` toward `look` by `dt` seconds, as far as the neck allows.
pub fn attend(m: &mut Motion, look: Option<Vec3>, dt: f32) {
    let root = (m.root.place, m.root.turn);
    let head = m.frame.roles.iter().position(|r| *r == Role::Head);
    let at = head.map_or(root.0, |i| carry(root, m.frame.rest[i]).0);
    let goal = aim(look, at, root.1);
    let spring = Spring::new(1.5 * m.tune.f, 0.75, 0.0);
    m.look.step(&spring, goal, dt);
}

/// Returns the sway of strand `index` of `kind` on `body` in `m` and its phase: a body wave down
/// the tail of a swimmer, a slow swing of each tentacle, a small swish of a walker's tail.
fn sway(m: &Motion, body: &Body, kind: Kind, index: usize) -> (Vec3, f32) {
    let right = m.root.turn * Vec3::X;
    let length = body.length.max(0.1);
    match (kind, body.locomotion) {
        (Kind::Tail, Locomotion::Swim) => (right * 0.15 * length, m.beat.cycle),
        (Kind::Tail, _) => (right * 0.04 * length, m.clock.phase + 0.25 * m.time),
        (Kind::Tentacle, _) => {
            let around = Quat::from_rotation_y(index as f32 * 2.4);
            (
                m.root.turn * (around * Vec3::X) * 0.08 * length,
                0.3 * m.time + 0.37 * index as f32,
            )
        }
        _ => (Vec3::ZERO, 0.0),
    }
}

/// Returns the spring a strand of `kind` lags on in `m`: stiff necks, loose tails, slow tentacles.
fn spring(m: &Motion, kind: Kind) -> Spring {
    match kind {
        Kind::Neck => Spring::new(2.5 * m.tune.f, 0.8, 0.0),
        Kind::Tail => Spring::new(1.6 * m.tune.f, 0.45, 0.0),
        _ => Spring::new(0.9 * m.tune.f, 0.35, 0.0),
    }
}

/// Lays the joints of a spine after its first along `ground`, as high over it as its anchor.
fn hug(points: &mut [Vec3], ground: &dyn Fn(Vec2) -> f32) {
    let Some(&first) = points.first() else {
        return;
    };
    let clear = first.y - ground(Vec2::new(first.x, first.z));
    for p in points.iter_mut().skip(1) {
        p.y = ground(Vec2::new(p.x, p.z)) + clear;
    }
}

/// Moves every chain of `body` in `m` after its body by `dt` seconds, over `ground`.
pub fn bend(m: &mut Motion, body: &Body, ground: &dyn Fn(Vec2) -> f32, dt: f32) {
    let root = (m.root.place, m.root.turn);
    let crawls = body.locomotion == Locomotion::Slither && m.detail == Some(Detail::Full);
    for (index, (kind, chain)) in chains(body).into_iter().enumerate() {
        let rest = m.frame.joints(chain, root);
        let (sway, phase) = sway(m, body, kind, index);
        let spring = spring(m, kind);
        let down = root.1 * Vec3::NEG_Y;
        let base = rest.get(1).map_or(down, |&next| unit(next - rest[0], down));
        let Some(strand) = m.strands.get_mut(index) else {
            continue;
        };
        if crawls && kind == Kind::Spine {
            hug(&mut strand.points, ground);
        }
        let pull = Pull {
            rest: &rest,
            base,
            sway,
            phase,
            spring,
        };
        strand.step(chain, &pull, dt);
    }
}
