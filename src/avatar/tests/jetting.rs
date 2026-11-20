//! A body the boot water jet pushes stops kicking: its legs stay still and straight.

use super::scene;
use crate::avatar::blend::pose_of;
use crate::avatar::clock::Clock;
use crate::avatar::pose::Limb;
use crate::avatar::{AvatarPose, Mode};

/// Returns the hip and knee angles of both legs in `mode` at the swim angle `stroke`.
fn legs(mode: Mode, thrust: f32, stroke: f32) -> [f32; 4] {
    let input = AvatarPose {
        speed: 3.0,
        thrust,
        ..scene(mode, 0.0)
    };
    let clock = Clock {
        stroke,
        ..Clock::default()
    };
    let pose = pose_of(mode, &input, &clock);
    [
        pose.limb(Limb::Hip, 0),
        pose.limb(Limb::Hip, 1),
        pose.limb(Limb::Knee, 0),
        pose.limb(Limb::Knee, 1),
    ]
}

/// Without the jet the legs kick through the stroke; at full jet they hold still and straight.
#[test]
fn the_water_jet_stills_the_kick() {
    for mode in [Mode::Swim, Mode::Dive] {
        let kicking = legs(mode, 0.0, 1.0);
        assert!(
            (kicking[0] - kicking[1]).abs() > 0.1,
            "{mode:?} kicks {kicking:?}"
        );
        let jetting = [0.0, 1.0, 2.5].map(|stroke| legs(mode, 1.0, stroke));
        for held in jetting {
            assert!(
                (held[0] - held[1]).abs() < 1e-4,
                "{mode:?} jet kicks {held:?}"
            );
            assert!(
                held[2].abs() < 1e-4 && held[3].abs() < 1e-4,
                "{mode:?} jet knees {held:?}"
            );
            assert!(
                (held[0] - jetting[0][0]).abs() < 1e-4,
                "{mode:?} legs move {held:?}"
            );
        }
    }
}
