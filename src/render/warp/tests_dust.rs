//! Warp dust tests: the flow follows the velocity, every grain flows out from the point ahead
//! frame by frame, and nothing pops when a grain wraps, the phase wraps or the field thins.

use bevy::prelude::*;

use super::dust::{DEPTH, FADE, LAPS, MOTES, TUBE, WRAP, crawl, hash, mote, window};
use super::{Drive, Lens, Sequence};

/// Frame time of the checks, in seconds.
const DT: f32 = 1.0 / 60.0;

/// Returns the state of a sequence held at `charge` and `velocity`.
fn held(charge: f32, velocity: f32) -> Sequence {
    Sequence {
        charge,
        velocity,
        ..Sequence::default()
    }
}

/// The flow is nothing outside a warp, creeps while the drive spools and rises with the
/// velocity to the top rate; the dust is out at rest and in through a warp.
#[test]
fn the_flow_rises_with_the_velocity() {
    assert_eq!(
        (Sequence::default().flow(), Sequence::default().dust()),
        (0.0, 0.0)
    );
    let spooled = held(0.45, 0.0);
    assert!(
        spooled.flow() > 0.0 && spooled.flow() < 0.1 * LAPS,
        "{}",
        spooled.flow()
    );
    let flows: Vec<f32> = (0..=20)
        .map(|i| held(1.0, i as f32 / 20.0).flow())
        .collect();
    assert!(flows.windows(2).all(|pair| pair[1] > pair[0]), "{flows:?}");
    assert_eq!(held(1.0, 1.0).flow(), LAPS);
    assert_eq!(held(1.0, 0.0).dust(), 1.0);
    let mut state = held(1.0, 1.0);
    for _ in 0..240 {
        state = state.advance(Drive::default(), DT);
    }
    assert_eq!((state.flow(), state.dust()), (0.0, 0.0));
}

/// The dust runs by time: the phase a lens carries is the same at 240 and at 20 frames a
/// second, stays put at rest and wraps inside its range.
#[test]
fn the_phase_runs_by_time_and_wraps() {
    let run = |dt: f32, seconds: f32| {
        let mut lens = held(1.0, 0.0).lens(Vec3::NEG_Z);
        for frame in 1..=(seconds / dt).round() as usize {
            let velocity = (frame as f32 * dt / seconds).min(1.0);
            lens = lens.carry(held(1.0, velocity).lens(Vec3::NEG_Z), dt);
        }
        lens.phase
    };
    let (fine, coarse) = (run(1.0 / 240.0, 6.0), run(1.0 / 20.0, 6.0));
    assert!(
        fine > 2.0 && (fine - coarse).abs() < 0.01,
        "{fine} {coarse}"
    );
    let rest = Lens::default().carry(Lens::default(), 10.0);
    assert_eq!(rest.phase, 0.0);
    let late = Lens {
        phase: WRAP - 0.01,
        flow: LAPS,
        ..Lens::default()
    };
    let wrapped = late.carry(late, 1.0).phase;
    assert!((0.0..WRAP).contains(&wrapped) && (wrapped - (LAPS - 0.01)).abs() < 1e-3);
}

/// Through a lap a grain comes from near the point ahead, swings out from it frame by frame
/// and passes behind the camera, and it is dark at both ends of the lap.
#[test]
fn a_grain_flows_out_from_the_point_ahead() {
    let step = LAPS * DT;
    for index in [0, 7, 513, MOTES - 1] {
        let start = 3.0 - hash(index * 4) + 1.0e-4;
        let frames = (1.0 / step) as usize - 1;
        let laps: Vec<_> = (0..frames)
            .map(|frame| mote(index, start + frame as f32 * step, 1.0, Vec3::NEG_Z))
            .collect();
        assert!(laps.windows(2).all(|pair| pair[1].lap > pair[0].lap));
        let angles: Vec<f32> = laps
            .iter()
            .map(|grain| grain.offset.angle_between(Vec3::NEG_Z))
            .collect();
        assert!(
            angles.windows(2).all(|pair| pair[1] > pair[0]),
            "{index}: {angles:?}"
        );
        assert!(angles[0] < TUBE.1 / DEPTH.0 * 1.01 && *angles.last().unwrap() > 1.6);
        let end = mote(index, start + 0.9998, 1.0, Vec3::NEG_Z);
        assert!(laps[0].light < 1e-3 && end.light < 1e-3, "{index}: {end:?}");
        assert!(laps.iter().any(|grain| grain.light > 0.01));
    }
}

/// No grain pops: while it is in view at the top flow its light changes between two frames by
/// at most an eighth of a full grain, the brightening of a grain that closes on the camera; it
/// changes by nothing across the wrap of its lap and of the phase, and by little while the field
/// thins out at the fastest rate a drop-out thins it.
#[test]
fn nothing_pops_frame_to_frame() {
    let step = LAPS * DT;
    let jump = |index: u32, phase: f32, density: (f32, f32), dp: f32| {
        let a = mote(index, phase, density.0, Vec3::NEG_Z).light;
        let b = mote(index, phase + dp, density.1, Vec3::NEG_Z).light;
        (a - b).abs()
    };
    for index in 0..MOTES {
        let phase = 40.0 + index as f32 * 0.37;
        let [early, late] = [phase, phase + WRAP].map(|at| mote(index, at, 1.0, Vec3::NEG_Z));
        let seen = early.offset.angle_between(Vec3::NEG_Z) < 1.0;
        let flowed = jump(index, phase, (1.0, 1.0), step);
        assert!(!seen || flowed < 0.12, "{index} flows: {flowed}");
        let edge = 6.0 - hash(index * 4);
        let lapped = jump(index, edge - step * 0.05, (1.0, 1.0), step * 0.1);
        assert!(lapped < 1e-3, "{index} laps: {lapped}");
        let wrapped = (early.offset - late.offset).length();
        assert!(wrapped < 1e-2, "{index} wraps: {wrapped}");
        assert!(
            jump(index, phase, (0.5, 0.5 - 1.5 * DT / 1.6), 0.0) < 0.05,
            "{index} thins"
        );
    }
}

/// The dust thins grain by grain to nothing as its share falls, and a lap fades in and out
/// over the shares it names.
#[test]
fn the_dust_thins_to_nothing() {
    let lit = |density: f32| {
        (0..MOTES)
            .filter(|index| mote(*index, 10.5, density, Vec3::NEG_Z).light > 0.0)
            .count()
    };
    let counts: Vec<usize> = [0.0, 0.25, 0.5, 0.75, 1.0].map(lit).to_vec();
    assert!(counts[0] == 0 && counts[1] > 0, "{counts:?}");
    assert!(
        counts.windows(2).all(|pair| pair[1] >= pair[0]),
        "{counts:?}"
    );
    assert!(counts[2] > counts[1] && counts[3] > counts[2], "{counts:?}");
    assert!(counts[4] as f32 > 0.9 * MOTES as f32);
    assert_eq!((window(0.0), window(1.0), window(0.5)), (0.0, 0.0, 1.0));
    assert!(window(FADE.0 * 0.5) < 1.0 && window(0.5 * (1.0 + FADE.1)) < 1.0);
}

/// The stars sway out along their streaks and back with no step: the sway is continuous over
/// its whole cycle and across the wrap, slow outward and quick back.
#[test]
fn the_stars_sway_without_a_step() {
    let cycle: Vec<f32> = (0..=1000).map(|i| crawl(i as f32 / 1000.0)).collect();
    assert!(
        cycle
            .windows(2)
            .all(|pair| (pair[1] - pair[0]).abs() < 0.01)
    );
    assert_eq!((crawl(0.0), crawl(1.0), crawl(0.8)), (0.0, 0.0, 1.0));
    assert!(cycle[..800].windows(2).all(|pair| pair[1] > pair[0]));
    assert!(cycle[800..].windows(2).all(|pair| pair[1] <= pair[0]));
}

/// The shader places the grains with the constants of the rules tested here.
#[test]
fn the_shader_has_the_constants_of_the_dust() {
    let shader = include_str!("../starfield.wesl");
    let lines = [
        format!("const MOTES: u32 = {MOTES}u;"),
        format!("const DEPTH_AHEAD: f32 = {:?};", DEPTH.0),
        format!("const DEPTH_BEHIND: f32 = {:?};", DEPTH.1),
        format!("const TUBE_NEAR: f32 = {:?};", TUBE.0),
        format!("const TUBE_FAR: f32 = {:?};", TUBE.1),
        format!("const FADE_IN: f32 = {:?};", FADE.0),
        format!("const FADE_OUT: f32 = {:?};", FADE.1),
        "let lap = u32(i32(floor(run)) & 1023);".to_string(),
        "let thin = clamp((stars.flow.z * 1.5 - hash(key + 3u)) * 2.0, 0.0, 1.0);".to_string(),
    ];
    for line in &lines {
        assert!(
            shader.contains(line.as_str()),
            "starfield.wesl lacks `{line}`"
        );
    }
    assert_eq!(WRAP, 1024.0);
}
