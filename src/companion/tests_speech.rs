//! Tests of the speech: over ten simulated minutes the companion speaks within its bounds at the
//! base, stays silent in flight but for warnings, always warns under water, and is silent when off.
//!
//! The simulation runs the real mind, face and speech, frame by frame, the way `think` chains them.

use super::face::Expression;
use super::face::machine::Face;
use super::idle::Idle;
use super::mind::{Input, Mind};
use super::mood::{ambient, chirp};
use super::sense::CompanionSense;
use super::speech::{Speech, Why, reason, spoken};
use super::tune::CHATTER_GAP;
use crate::settings::Chatter;

/// Frame time of the simulation.
const DT: f32 = 1.0 / 30.0;

/// Seconds simulated.
const SPAN: f32 = 600.0;

/// What ten minutes of play heard.
#[derive(Debug, Default)]
struct Heard {
    /// Chirps the speech let through.
    sent: usize,
    /// Of them, chatter.
    chatter: usize,
    /// Of them, warnings.
    warnings: usize,
    /// Chirps the companion would have sent with no speech at all.
    wanted: usize,
}

/// Runs ten minutes of a player whose speed and surroundings at second `t` are `play(t)`, with the
/// voice setting `voice`, and returns what was heard.
fn run(play: impl Fn(f32) -> (f32, CompanionSense), voice: Chatter) -> Heard {
    let (mut mind, mut face, mut speech) = (Mind::new(3), Face::new(3), Speech::default());
    let mut heard = Heard::default();
    for frame in 0..(SPAN / DT) as usize {
        let (speed, sense) = play(frame as f32 * DT);
        let input = Input {
            speed,
            night: false,
            seated: sense.seated,
            dt: DT,
        };
        let cue = mind.advance(&input);
        let idle = match mind.dozing {
            true => Some(Expression::Sleepy),
            false => mind.idle.map(Idle::face),
        };
        let changed = face.advance(ambient(&sense, idle), DT);
        let said = cue.and_then(spoken).or(changed
            .and_then(chirp)
            .map(|kind| (kind, reason(kind, &sense))));
        heard.wanted += usize::from(said.is_some());
        let aloft = sense.flying || sense.seated;
        if speech.hear(said, (voice, aloft), DT).is_some() {
            let why = said.map(|(_, why)| why);
            heard.sent += 1;
            heard.chatter += usize::from(why == Some(Why::Chatter));
            heard.warnings += usize::from(why == Some(Why::Warning));
        }
    }
    heard
}

/// Returns the base: a player who walks for 20 seconds, then stands for a minute, over and over.
fn base(t: f32) -> (f32, CompanionSense) {
    let speed = if t % 80.0 < 20.0 { 1.5 } else { 0.0 };
    (speed, CompanionSense::default())
}

/// At the base the companion greets the player as it wakes and chatters no more than once every
/// [`CHATTER_GAP`] seconds, when the face alone would chirp many times as often.
#[test]
fn the_base_is_quiet() {
    let rare = run(base, Chatter::Rare);
    let often = run(base, Chatter::Often);
    println!("base, ten minutes: {rare:?} rare, {often:?} often");
    assert!(
        rare.chatter <= (SPAN / CHATTER_GAP.1) as usize + 1,
        "{rare:?}"
    );
    assert!(
        often.chatter <= (SPAN / CHATTER_GAP.0) as usize + 1,
        "{often:?}"
    );
    assert!(rare.sent <= 12 && rare.sent > 0, "{rare:?}");
    assert!(rare.wanted > 2 * rare.sent, "{rare:?}");
}

/// Flying with the gear through dives, or seated in a climbing ship through the clouds, the companion
/// says nothing: there is no danger, only changes of face.
#[test]
fn flight_is_silent_but_for_warnings() {
    let gear = |t: f32| {
        let sense = CompanionSense {
            flying: true,
            diving: t % 30.0 < 5.0,
            ..CompanionSense::default()
        };
        (30.0, sense)
    };
    let ship = |t: f32| {
        let sense = CompanionSense {
            seated: true,
            climb: if t < 20.0 { 12.0 } else { 0.0 },
            in_clouds: (40.0..60.0).contains(&(t % 120.0)),
            ..CompanionSense::default()
        };
        (0.0, sense)
    };
    let (a, b) = (run(gear, Chatter::Rare), run(ship, Chatter::Often));
    println!("flight, ten minutes: {a:?} with the gear, {b:?} in the ship");
    assert_eq!((a.sent, b.sent), (0, 0), "{a:?} {b:?}");
    assert!(a.wanted > 10, "{a:?}");
}

/// Under water the companion warns every time, however rarely it chatters, but never twice in a
/// row within the pause between warnings.
#[test]
fn water_over_the_head_is_warned() {
    let dives = |t: f32| {
        let sense = CompanionSense {
            depth: if t % 60.0 < 10.0 { 1.0 } else { 0.0 },
            ..CompanionSense::default()
        };
        (0.0, sense)
    };
    let heard = run(dives, Chatter::Rare);
    println!("ten dives in ten minutes: {heard:?}");
    assert!((8..=10).contains(&heard.warnings), "{heard:?}");
}

/// Turned off, the companion says nothing at all, not even a warning.
#[test]
fn off_is_silent() {
    let heard = run(base, Chatter::Off);
    assert_eq!(heard.sent, 0, "{heard:?}");
    assert!(heard.wanted > 0);
}
