//! The voice of a ship in the mix: no step when its state jumps, the same samples every run,
//! mechanisms that are heard, and a cost that stays far under real time.

use std::time::Instant;

use super::rig::{alone, heard, parked, rms, seated};
use super::{RATE, Stats, frames, listen, mixer};
use crate::audio::cue::AudioCue;
use crate::audio::mix::BLOCK;
use crate::audio::ship::Fleet;
use crate::audio::ship::feed::{Ch, Deck, HULLS, HullSound};
use crate::settings::Audio as Volumes;

/// Largest step between two samples the seat may hear when the state of the ship jumps. The
/// hardest thump of a lock climbs by a tenth of this per sample.
const STEP: f32 = 0.12;

/// States a ship can jump between from one frame to the next, as far apart as they get below
/// the thrust where a jet crackles. The canopy opens from idle, as it does in the game: opened
/// in the middle of a burn it lets the roar in, which is loud and is not a step.
fn jumps() -> [HullSound; 6] {
    let idle = parked();
    [
        idle.with(Ch::Thrust, 0.55).with(Ch::Lift, 1.0),
        idle,
        idle.with(Ch::Gear, 0.0).with(Ch::Canopy, 1.0),
        idle.with(Ch::Power, 0.0).with(Ch::Air, 0.0),
        idle.with(Ch::Warp, 1.0)
            .with(Ch::Rcs, 1.0)
            .with(Ch::Glow, 1.0),
        idle.with(Ch::Speed, 120.0).with(Ch::Rough, 1.0),
    ]
}

/// The state of the ship may jump from any value to any other between two frames: every value
/// is smoothed, so the seat hears no step, and neither does a listener who stands 60 metres off
/// or one who gets in and out. The listener stays in the seat the round after a burn: one who
/// gets out of a burn hears its roar 60 metres off, whose own noise moves more than a step per
/// sample. [`getting_out_of_a_burn_sweeps_nothing`] holds that case.
#[test]
fn a_jump_of_the_state_makes_no_step() {
    let (mut mixer, shared) = mixer(&alone(seated(parked())));
    let steady = listen(&mut mixer, 3.0);
    assert!(steady.jump < STEP, "steady {}", steady.jump);
    for (round, hull) in jumps().into_iter().cycle().take(18).enumerate() {
        let seat = round % 5 < 3 || round % 6 == 1;
        let hull = if seat { seated(hull) } else { hull };
        shared.publish(&alone(hull), &Volumes::default());
        let stats = listen(&mut mixer, 0.7);
        assert!(
            !stats.broken && stats.jump < STEP,
            "{round}: {}",
            stats.jump
        );
    }
}

/// A listener who gets out of a ship in the middle of a burn hears the roar on its way from
/// where they stand, no rougher than that burn heard there all along. A delay that glided from
/// the seat out to them would play it at up to twice its pitch, which is rougher.
#[test]
fn getting_out_of_a_burn_sweeps_nothing() {
    let burn = parked().with(Ch::Thrust, 0.55).with(Ch::Lift, 1.0);
    let (mut there, _) = mixer(&alone(burn));
    listen(&mut there, 3.0);
    let steady = listen(&mut there, 2.0);
    let (mut pilot, shared) = mixer(&alone(parked()));
    listen(&mut pilot, 3.0);
    shared.publish(&alone(seated(burn)), &Volumes::default());
    listen(&mut pilot, 2.0);
    shared.publish(&alone(parked()), &Volumes::default());
    let out = listen(&mut pilot, 0.7);
    assert!(
        !out.broken && out.jump < steady.jump,
        "{} {}",
        out.jump,
        steady.jump
    );
}

/// The same scene renders the same samples every time: every random number is seeded.
#[test]
fn the_voice_is_deterministic() {
    let hull = seated(parked().with(Ch::Thrust, 1.3).with(Ch::Rough, 0.7));
    let first = heard(hull, 1.5);
    assert_eq!(first, heard(hull, 1.5));
    assert!(rms(&first) > 0.01);
}

/// Renders `seconds` of a seated ship whose gear goes from `from` to `to` at the start.
fn gear(from: f32, to: f32, seconds: f32) -> Stats {
    let (mut mixer, shared) = mixer(&alone(seated(parked().with(Ch::Gear, from))));
    listen(&mut mixer, 6.0);
    shared.publish(
        &alone(seated(parked().with(Ch::Gear, to))),
        &Volumes::default(),
    );
    listen(&mut mixer, seconds)
}

/// The gear is heard as it arrives: its lock thumps well over the idle of the ship. A landing
/// thumps too, in a ship that is shut down, and harder when it is hard.
#[test]
fn locks_and_landings_thump() {
    let still = gear(1.0, 1.0, 1.0);
    let locked = gear(0.0, 1.0, 1.0);
    assert!(
        locked.peak > 1.5 * still.peak,
        "{} {}",
        locked.peak,
        still.peak
    );
    let landing = |strength: f32| {
        let (mut mixer, shared) = mixer(&alone(seated(parked().with(Ch::Power, 0.0))));
        listen(&mut mixer, 6.0);
        shared.push(AudioCue::Land(strength));
        listen(&mut mixer, 1.0).peak
    };
    assert!(landing(1.0) > landing(0.1));
    assert!(landing(0.1) > 0.04);
}

/// Three ships at their loudest, the most there can be, render faster than real time even in a
/// test build, which is several times slower than the game: the audio thread is never short of
/// time because of the ships. The measured factor is printed.
#[test]
fn the_ships_cost_little() {
    let loud = parked().with(Ch::Thrust, 1.7).with(Ch::Lift, 1.0);
    let deck = Deck::default();
    deck.publish(&[seated(loud).with(Ch::Warp, 1.0), loud, loud]);
    let dt = BLOCK as f32 / RATE;
    let mut fleet = Fleet::new(RATE, dt);
    let seconds = 4.0;
    let clock = Instant::now();
    let mut peak = 0.0_f32;
    for i in 0..frames(seconds) {
        if i % BLOCK as usize == 0 {
            fleet.tune(&deck, 0.0, dt);
        }
        peak = peak.max(fleet.frame(0.0)[0].abs());
    }
    let spent = clock.elapsed().as_secs_f32();
    println!("{HULLS} ships: {:.1} times real time", seconds / spent);
    assert!(peak > 0.05 && peak <= 0.5);
    assert!(spent < seconds, "{spent} s for {seconds} s of sound");
}
