//! The music stays clean, deterministic and silent where it must be.

use super::{CLICK, RATE, Stats, frames, listen, mixer, planet};
use crate::audio::dsp::limit::KNEE;
use crate::audio::mix::BLOCK;
use crate::audio::music::{Music, MusicCtl};
use crate::audio::scene::AudioScene;
use crate::settings::Audio as Volumes;

/// Renders `seconds` of the bare music engine for a planet, at full level by day.
fn play(seed: u64, home: bool, seconds: f32) -> Vec<[f32; 2]> {
    let mut music = Music::new(RATE, seed, home);
    let orders = MusicCtl {
        seed,
        home,
        level: 1.0,
        day: 1.0,
        submerged: 0.0,
    };
    let frame = |i: usize| {
        if i.is_multiple_of(BLOCK as usize) {
            music.steer(&orders, BLOCK as f32 / RATE);
        }
        music.frame()
    };
    (0..frames(seconds)).map(frame).collect()
}

/// Runs `job` for twenty seeds at once and returns the results in seed order.
fn for_seeds<T: Send>(job: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let job = &job;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (1..=20)
            .map(|seed| scope.spawn(move || job(seed)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// Sixty seconds of twenty planets: in range, finite, centred, audible and free of clicks,
/// from the first sample through every chord change.
#[test]
fn music_is_bounded_centred_and_click_free() {
    let all = for_seeds(|seed| {
        let (mut mixer, _shared) = mixer(&planet(seed * 1_000_003));
        let first = mixer.frame();
        (first, listen(&mut mixer, 60.0))
    });
    for (first, stats) in all {
        assert!(!stats.broken);
        assert!(stats.peak <= 1.0);
        assert!(first[0].abs() < 1e-3 && first[1].abs() < 1e-3);
        assert!(stats.jump < CLICK, "jump {}", stats.jump);
        assert!(stats.offset() < 1e-3, "offset {}", stats.offset());
        assert!(stats.rms() > 0.01, "rms {}", stats.rms());
    }
}

/// The bare music never reaches the knee of the limiter, so the limiter never colours it.
#[test]
fn music_keeps_headroom() {
    let peaks = for_seeds(|seed| {
        let buffer = play(seed * 77 + 5, seed % 5 == 0, 45.0);
        buffer.into_iter().fold(Stats::default(), Stats::add).peak
    });
    for peak in peaks {
        assert!(peak < KNEE, "peak {peak}");
        assert!(peak > 0.02, "peak {peak}");
    }
}

/// The same planet renders bit-equal twice, another planet differs, and so does the home variant.
#[test]
fn same_seed_same_sound() {
    let once = play(42, false, 6.0);
    assert!(once == play(42, false, 6.0));
    assert!(once != play(43, false, 6.0));
    assert!(once != play(42, true, 6.0));
}

/// In space and in warp the music fades to exact silence within seconds, and comes back after.
#[test]
fn music_is_silent_in_space() {
    for away in [
        AudioScene {
            in_space: true,
            ..planet(9)
        },
        AudioScene {
            warping: true,
            ..planet(9)
        },
    ] {
        let (mut mixer, shared) = mixer(&planet(9));
        assert!(listen(&mut mixer, 6.0).rms() > 0.005);
        shared.publish(&away, &Volumes::default());
        let fading = listen(&mut mixer, 9.0);
        assert!(fading.jump < CLICK);
        assert_eq!(listen(&mut mixer, 2.0).peak, 0.0);
        shared.publish(&planet(9), &Volumes::default());
        let back = listen(&mut mixer, 6.0);
        assert!(back.rms() > 0.005 && back.jump < CLICK);
    }
}

/// Landing on another planet swaps the piece while silent: no click, and the new piece plays.
#[test]
fn planet_change_never_clicks() {
    let (mut mixer, shared) = mixer(&planet(3));
    listen(&mut mixer, 5.0);
    shared.publish(&planet(4), &Volumes::default());
    let change = listen(&mut mixer, 20.0);
    assert!(!change.broken && change.jump < CLICK);
    let mut music = Music::new(RATE, 3, false);
    let orders = MusicCtl {
        seed: 4,
        home: false,
        level: 1.0,
        day: 1.0,
        submerged: 0.0,
    };
    for _ in 0..frames(1.0) / BLOCK as usize {
        music.steer(&orders, BLOCK as f32 / RATE);
        music.frame();
    }
    assert_eq!(music.piece().seed, 4);
}

/// Night and water darken the music without breaking it, and night is quieter than day.
#[test]
fn night_and_water_stay_clean() {
    let scene = |daylight: f32, submerged: f32| AudioScene {
        daylight,
        submerged,
        ..planet(11)
    };
    let level = |scene: AudioScene| {
        let (mut mixer, _shared) = mixer(&scene);
        let stats = listen(&mut mixer, 30.0);
        assert!(!stats.broken && stats.jump < CLICK && stats.offset() < 1e-3);
        stats.rms()
    };
    let (day, night, deep) = (
        level(scene(1.0, 0.0)),
        level(scene(0.0, 0.0)),
        level(scene(1.0, 1.0)),
    );
    assert!(night < day && night > 0.2 * day);
    assert!(deep > 0.0);
}
