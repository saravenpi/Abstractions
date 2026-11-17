//! The wingsuit and the boot plumes: the unfold, the fabric between the joints, and the glow.

use bevy::prelude::*;

use super::{FRAME, scene};
use crate::avatar::Mode;
use crate::avatar::blend::pose_of;
use crate::avatar::fx::{plume_color, plume_length};
use crate::avatar::rig::{Seg, globals};
use crate::avatar::tune::{PLUME, UNFOLD_SECONDS};
use crate::avatar::wing::{Panel, fan, flutter, rim, stretch, unfold_step};

/// Returns the area of a panel in square meters.
fn area(panel: Panel, open: f32) -> f32 {
    let frames = globals(&pose_of(Mode::Glide, &scene(Mode::Glide, 0.5), &default()));
    let triangles = fan(&rim(panel, &frames, open), 0.0);
    triangles
        .iter()
        .map(|[a, b, c]| (*b - *a).cross(*c - *a).length() * 0.5)
        .sum()
}

/// The unfold only ever grows, takes the documented time, and folds back the same way.
#[test]
fn unfold_is_monotone_and_takes_the_documented_time() {
    let (mut unfold, mut frames) = (0.0_f32, 0);
    while unfold < 1.0 {
        let next = unfold_step(unfold, 1.0, FRAME);
        assert!(
            next > unfold && next <= 1.0,
            "unfold went from {unfold} to {next}"
        );
        assert!(stretch(next) >= stretch(unfold));
        (unfold, frames) = (next, frames + 1);
    }
    let seconds = frames as f32 * FRAME;
    assert!(
        (seconds - UNFOLD_SECONDS).abs() <= FRAME,
        "unfolded in {seconds} s"
    );
    assert_eq!(unfold_step(1.0, 1.0, FRAME), 1.0);
    while unfold > 0.0 {
        let next = unfold_step(unfold, 0.0, FRAME);
        assert!(next < unfold && next >= 0.0);
        unfold = next;
    }
    assert_eq!((stretch(0.0), stretch(1.0)), (0.0, 1.0));
    assert_eq!(
        unfold_step(0.4, f32::NAN, FRAME),
        0.4 - FRAME / UNFOLD_SECONDS
    );
    assert_eq!(unfold_step(0.0, 7.0, 10.0), 1.0);
}

/// Open, the fabric spans from the arms to the body and between the legs. Folded, it has next to no area.
#[test]
fn fabric_stretches_between_the_joints() {
    for panel in Panel::ALL {
        assert!(
            area(panel, 0.0) < 0.005,
            "{panel:?} folded covers {}",
            area(panel, 0.0)
        );
        assert!(
            area(panel, 1.0) > 0.06,
            "{panel:?} open covers {}",
            area(panel, 1.0)
        );
        assert!(area(panel, 0.5) > area(panel, 0.1) && area(panel, 1.0) > area(panel, 0.5));
    }
    let frames = globals(&pose_of(Mode::Glide, &scene(Mode::Glide, 0.5), &default()));
    let right = rim(Panel::ArmR, &frames, 1.0);
    assert_eq!(right[2], frames[Seg::HandR as usize].translation);
    assert_eq!(right[1], frames[Seg::ForearmR as usize].translation);
    assert!(
        right.iter().all(|p| p.x > 0.0),
        "right panel crosses the body: {right:?}"
    );
    let legs = rim(Panel::Legs, &frames, 1.0);
    assert_eq!(legs[2], frames[Seg::BootL as usize].translation);
    assert_eq!(legs[3], frames[Seg::BootR as usize].translation);
}

/// The flutter grows with the airspeed, stays gentle, and a folded panel does not flutter.
#[test]
fn flutter_grows_with_airspeed() {
    let (slow, fast) = (flutter(5.0, 1.0, 1.0), flutter(40.0, 1.0, 1.0));
    assert!(
        slow > 0.0 && fast > 2.0 * slow && fast < 0.03,
        "{slow} {fast}"
    );
    assert_eq!(flutter(40.0, 1.0, 0.0), 0.0);
    assert_eq!(flutter(40.0, -1.0, 1.0), -fast);
    assert!(flutter(500.0, 1.0, 1.0) <= flutter(45.0, 1.0, 1.0) + 1e-6);
}

/// The plume lengthens and brightens with the thrust, above one at full thrust so it blooms.
#[test]
fn plume_follows_the_thrust() {
    let bright = |thrust: f32| plume_color(thrust).to_linear().blue;
    for pair in [0.0, 0.25, 0.5, 0.75, 1.0].windows(2) {
        assert!(plume_length(pair[1]) > plume_length(pair[0]));
        assert!(bright(pair[1]) > bright(pair[0]));
    }
    assert_eq!((plume_length(-3.0), plume_length(9.0)), PLUME);
    assert!(bright(1.0) > 4.0 && plume_color(1.0).to_linear().green > 1.0);
    assert!(
        bright(0.0) < 0.5,
        "the resting nozzle glares: {}",
        bright(0.0)
    );
}
