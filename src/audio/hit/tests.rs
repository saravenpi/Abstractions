//! Tests of the sound of blows: louder with energy, a plate brighter than flesh, wet only on a cut,
//! a heavy tool lower and louder in the air, and the packing.

use bevy::prelude::*;

use super::sound::{contact, loudness};
use super::{Body, HitSound, Impact, Kind, Outer, Whoosh, of_impact, of_whoosh};
use crate::audio::cue::AudioCue;
use crate::audio::cues::Pool;
use crate::audio::spatial::Spot;

/// A stone head of 0.3 kg.
const STONE: Body = Body {
    stiffness: 30.0,
    mass: 0.3,
};

/// Returns a blow of `joules` on `struck` through `outer`.
fn blow(joules: f32, outer: Outer, struck: Body, cut: bool) -> Impact {
    Impact {
        at: Vec3::ZERO,
        joules,
        cut,
        outer,
        struck,
        tool: STONE,
        deflected: false,
    }
}

/// Flesh and a plate, as bodies.
const FLESH: Body = Body {
    stiffness: 0.1,
    mass: 50.0,
};
const PLATE: Body = Body {
    stiffness: 17.0,
    mass: 2.0,
};

/// The loudness follows the logarithm of the energy and stays within 0 and 1.
#[test]
fn loudness_grows_with_the_energy() {
    assert_eq!(loudness(0.0), 0.0);
    assert!(loudness(1.0) < loudness(10.0) && loudness(10.0) < loudness(100.0));
    assert!((loudness(1.0e6) - 1.0).abs() < 1e-6);
    let soft = of_impact(&blow(2.0, Outer::Hide, FLESH, false));
    let hard = of_impact(&blow(200.0, Outer::Hide, FLESH, false));
    assert!(hard.level > soft.level);
}

/// A plate rings higher and brighter than flesh, and a soft body deadens a hard one.
#[test]
fn a_plate_is_brighter_than_flesh() {
    let flesh = of_impact(&blow(50.0, Outer::Flesh, FLESH, false));
    let plate = of_impact(&blow(50.0, Outer::RobotPlate, PLATE, false));
    assert!(plate.hard > flesh.hard + 0.4 && plate.hz > flesh.hz);
    assert!(contact(0.1, 30.0) < 0.11 && contact(17.0, 30.0) > 10.0);
}

/// The wet noise comes only when the blow cut, and a glance is a tick without it.
#[test]
fn wet_only_when_it_cut() {
    let mut cut = blow(50.0, Outer::Flesh, FLESH, true);
    assert!(of_impact(&cut).wet);
    cut.cut = false;
    assert!(!of_impact(&cut).wet);
    cut.cut = true;
    cut.deflected = true;
    let tick = of_impact(&cut);
    assert!(!tick.wet && tick.kind == super::Kind::Tick);
}

/// A heavy tool whooshes louder and lower than a light one at the same speed.
#[test]
fn a_heavy_tool_whooshes_lower_and_louder() {
    let swing = |inertia| Whoosh {
        at: Vec3::ZERO,
        speed: 10.0,
        inertia,
    };
    let (light, heavy) = (of_whoosh(&swing(0.02)), of_whoosh(&swing(0.6)));
    assert!(heavy.level > light.level && heavy.hz < light.hz);
    assert!(
        of_whoosh(&Whoosh {
            speed: 15.0,
            ..swing(0.02)
        })
        .level
            > light.level
    );
}

/// A sound survives its packing to within a step, and every sound renders within range and on at most
/// three voices.
#[test]
fn sounds_pack_and_render() {
    let sounds = [
        of_impact(&blow(500.0, Outer::RobotPlate, PLATE, true)),
        of_impact(&Impact {
            deflected: true,
            ..blow(40.0, Outer::Horn, PLATE, false)
        }),
        of_whoosh(&Whoosh {
            at: Vec3::ZERO,
            speed: 12.0,
            inertia: 0.3,
        }),
    ];
    for sound in sounds {
        let back = super::HitSound::unpack(sound.pack()).expect("packed");
        assert_eq!(back.kind, sound.kind);
        assert!((back.level - sound.level).abs() < 0.07);
        assert!(back.hz / sound.hz < 1.1 && sound.hz / back.hz < 1.1);
        let mut pool = Pool::new(48_000.0);
        pool.play(AudioCue::Hit(back), &Spot::FLAT);
        let peak = (0..48_000)
            .map(|_| pool.frame()[0].abs())
            .fold(0.0_f32, f32::max);
        assert!(peak > 0.01 && peak < 0.9, "{sound:?} {peak}");
    }
}

/// A gunshot keeps its kind, wet flag, and quantized fields across pack and unpack.
#[test]
fn shot_packs_and_unpacks() {
    for (level, hz, hard) in [(1.0, 92.0, 0.9), (0.85, 135.0, 0.65)] {
        let sound = HitSound {
            kind: Kind::Shot,
            level,
            hz,
            hard,
            wet: false,
        };
        let back = HitSound::unpack(sound.pack()).expect("packed shot");
        assert_eq!(back.kind, Kind::Shot);
        assert!(!back.wet);
        assert!((back.level - sound.level).abs() < 0.07);
        assert!(back.hz / sound.hz < 1.1 && sound.hz / back.hz < 1.1);
        assert!((back.hard - sound.hard).abs() < 0.15);
        assert_eq!(HitSound::unpack(back.pack()), Some(back));
    }
}

/// A gunshot cue uses three layered blips and renders within bounds.
#[test]
fn shot_renders_three_blips() {
    for (level, hz, hard) in [(1.0, 92.0, 0.9), (0.85, 135.0, 0.65)] {
        let sound = HitSound {
            kind: Kind::Shot,
            level,
            hz,
            hard,
            wet: false,
        };
        let mut pool = Pool::new(48_000.0);
        let before = pool.free();
        pool.play(AudioCue::Hit(sound), &Spot::FLAT);
        assert_eq!(before - pool.free(), 3);
        let peak = (0..48_000)
            .map(|_| pool.frame()[0].abs())
            .fold(0.0_f32, f32::max);
        assert!(peak > 0.01 && peak < 0.9, "{sound:?} {peak}");
    }
}
