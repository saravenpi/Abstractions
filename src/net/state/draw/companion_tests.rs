//! The pose of a remote companion follows the two records around the time it is drawn at.

use abstractions_net::jitter::View;
use abstractions_net::wire::CompanionBody;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::companion::posed;

/// A record whose pose fields all hold `value`.
fn record(value: f32) -> CompanionBody {
    CompanionBody {
        lift: value,
        yaw: value,
        pitch: value,
        roll: value,
        spread: value,
        wave: value,
        ..CompanionBody::default()
    }
}

/// Halfway between two records, every pose field is halfway between theirs.
#[test]
fn the_pose_blends_between_records() {
    let view = View {
        time: 0.0,
        position: DVec3::ZERO,
        rotation: Quat::IDENTITY,
        older: record(0.2),
        newer: record(0.6),
        blend: 0.5,
    };
    let pose = posed(&view);
    for field in [
        pose.lift,
        pose.yaw,
        pose.pitch,
        pose.roll,
        pose.spread,
        pose.wave,
    ] {
        assert!((field - 0.4).abs() < 1e-6, "{field}");
    }
}
