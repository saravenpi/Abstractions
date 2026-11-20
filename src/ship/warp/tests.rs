//! Warp tests on the real drive: its speed scale, and the sky of a whole warp to the home
//! planet, where the destination must grow into view without a step.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{
    CRUISE_CAP, FLOOR, GALAXY_CAP, JUMP_CAP, SPOOL_TIME, Warp, advance, fraction, request,
};
use crate::render::{Drive, Phase, Sequence};
use crate::ship::body::Ship;
use crate::ship::fx::{pace, warp_level};
use crate::ship::telemetry::WarpState;
use crate::ship::wells::{Well, system_wells};
use crate::space::Frame;
use crate::universe::CARRY;
use crate::universe::{SystemSpec, Universe};

/// The stages a whole warp goes through.
const STAGES: [Phase; 5] = [
    Phase::Spool,
    Phase::Engage,
    Phase::Cruise,
    Phase::DropOut,
    Phase::Idle,
];

/// Seconds of one frame of the test warps.
const DT: f64 = 1.0 / 30.0;

/// Returns the home system, the rank of the home planet and a ship at rest relative to it,
/// `radii` planet radii out on the side away from the star, nose toward the planet.
fn bound_home(radii: f64) -> (SystemSpec, usize, Ship) {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let index = usize::from(universe.home().index);
    let well = Well::planet(&system.planets[index], 0.0);
    let out = well.pos.normalize();
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, -out);
    let pos = well.pos + out * (well.radius * radii);
    let mut ship = Ship::at_rest(Frame::System(system.star.id), pos, rot);
    ship.vel = well.vel;
    (system, index, ship)
}

/// Returns what the sky is told of a drive, through what the instruments say of it.
fn shown(warp: Warp) -> Drive {
    let state = match warp {
        Warp::Off => WarpState::Off,
        Warp::Spooling { left } => WarpState::Spooling(1.0 - left / SPOOL_TIME),
        Warp::Active { speed } => WarpState::Active(speed),
    };
    Drive {
        charge: warp_level(state),
        speed: pace(state),
    }
}

/// One frame of a warp as the sky draws it.
struct Frame30 {
    /// The stage of the sequence.
    phase: Phase,
    /// Angular radius in radians the destination is seen with.
    angle: f32,
    /// Radians the destination is seen farther from the heading than it truly is.
    off: f64,
    /// True angular radius of the destination.
    truth: f32,
}

/// Flies `ship` at planet `index` of `system` until the drive has stopped and the sequence is
/// at rest again, and returns every frame.
fn fly_home(system: &SystemSpec, index: usize, ship: &mut Ship) -> Vec<Frame30> {
    let (mut frames, mut sequence, mut time) = (Vec::new(), Sequence::default(), 0.0);
    while (ship.warp != Warp::Off || sequence.phase != Phase::Idle) && time < 600.0 {
        let wells = system_wells(system, time);
        let home = wells[index + 1];
        ship.rot = DQuat::from_rotation_arc(DVec3::NEG_Z, (home.pos - ship.pos).normalize());
        advance(ship, &wells, CRUISE_CAP, DT);
        time += DT;
        sequence = sequence.advance(shown(ship.warp), DT as f32);
        let offset = Well::planet(&system.planets[index], time).pos - ship.pos;
        let seen = sequence
            .lens(ship.forward().as_vec3())
            .body(offset, home.radius);
        frames.push(Frame30 {
            phase: sequence.phase,
            angle: seen.angle(home.radius),
            off: seen.offset.angle_between(ship.forward()) - offset.angle_between(ship.forward()),
            truth: (home.radius / offset.length()) as f32,
        });
    }
    frames
}

/// The share of the top speed runs from the slowest warp to the fastest jump on a log scale:
/// a tenfold speed is always the same step, and every tier sits at the share its cap gives.
#[test]
fn speed_share_spans_the_drive_on_a_log_scale() {
    assert_eq!(
        (fraction(0.0), fraction(FLOOR), fraction(GALAXY_CAP)),
        (0.0, 0.0, 1.0)
    );
    assert_eq!(fraction(GALAXY_CAP * 10.0), 1.0);
    let decades: Vec<f64> = (4..=13).map(|power| fraction(10f64.powi(power))).collect();
    for pair in decades.windows(2) {
        assert!((pair[1] - pair[0] - decades[1] + decades[0]).abs() < 1e-12);
        assert!(pair[1] > pair[0]);
    }
    let share = |cap: f64| (cap / FLOOR).ln() / (GALAXY_CAP / FLOOR).ln();
    assert!((fraction(CRUISE_CAP) - share(CRUISE_CAP)).abs() < 1e-12);
    assert!((fraction(CRUISE_CAP) - 0.73).abs() < 0.01);
    assert!((fraction(JUMP_CAP) - 0.92).abs() < 0.01);
    assert!(fraction(CRUISE_CAP) < fraction(JUMP_CAP));
}

/// The drive carries a body in the bubble inside its Hill sphere, a cruise between the stars
/// keeps every single star in sight, and a jump blurs them.
#[test]
fn the_sky_follows_the_scales_of_the_drive() {
    assert!(0.0 < CARRY.0 && CARRY.0 < CARRY.1 && CARRY.1 <= 1.0);
    let settled = |speed: f64| {
        let engaged = shown(Warp::Active { speed });
        (0..600).fold(Sequence::default(), |state, _| state.advance(engaged, 0.1))
    };
    let (cruise, jump) = (settled(CRUISE_CAP), settled(GALAXY_CAP));
    assert_eq!((cruise.phase, cruise.stars()), (Phase::Cruise, 1.0));
    assert_eq!((jump.velocity, jump.stars()), (1.0, 0.25));
    assert_eq!(settled(JUMP_CAP).stars(), 0.25);
    assert!(jump.beta() > cruise.beta() && cruise.beta() > settled(FLOOR).beta());
    assert_eq!(shown(Warp::Off), Drive::default());
    assert!(shown(Warp::Spooling { left: 0.0 }).charge < 1.0);
}

/// On a warp to the home planet the destination is never pushed off the heading and its seen
/// size never steps: not while the ship closes, not when the planet enters the bubble, not
/// across the drop-out. It ends at its true size with the drive off.
#[test]
fn the_destination_grows_without_a_step() {
    let (system, index, mut ship) = bound_home(4000.0);
    assert!(request(&mut ship, &system_wells(&system, 0.0)));
    let frames = fly_home(&system, index, &mut ship);
    let stages: Vec<Phase> = frames.iter().map(|frame| frame.phase).collect();
    for stage in STAGES {
        assert!(stages.contains(&stage), "the warp never reached {stage:?}");
    }
    assert_eq!((ship.warp, stages.last()), (Warp::Off, Some(&Phase::Idle)));
    for pair in frames.windows(2) {
        let ratio = pair[1].angle / pair[0].angle;
        assert!((0.93..1.07).contains(&ratio), "the size stepped by {ratio}");
        assert!(pair[1].off < 1e-6, "pushed off the heading");
    }
    let (spool, last) = (&frames[60], frames.last().unwrap());
    assert!(spool.angle < spool.truth, "the sky ahead does not shrink");
    assert!(last.angle == last.truth && last.angle > 40.0 * spool.angle);
    let cruising = |frame: &&Frame30| frame.phase == Phase::Cruise;
    let cruise = frames.iter().find(cruising).unwrap();
    assert!(cruise.angle < 0.6 * cruise.truth);
}

/// A drive stopped in the middle of a warp leaves the sky to drop out from the state it has:
/// the seen size of the planet ahead eases back to the true one without a step.
#[test]
fn an_interrupted_warp_drops_out_from_where_it_is() {
    let (system, index, mut ship) = bound_home(4000.0);
    let wells = system_wells(&system, 0.0);
    let home = wells[index + 1];
    assert!(request(&mut ship, &wells));
    let (mut sequence, mut scales) = (Sequence::default(), Vec::new());
    for frame in 0..400 {
        if frame == 200 {
            super::drop_out(&mut ship, &wells);
        }
        advance(&mut ship, &wells, CRUISE_CAP, DT);
        sequence = sequence.advance(shown(ship.warp), DT as f32);
        let lens = sequence.lens(ship.forward().as_vec3());
        scales.push(lens.body(home.pos - ship.pos, home.radius).scale);
        assert_eq!(
            frame >= 200,
            matches!(sequence.phase, Phase::DropOut | Phase::Idle)
        );
    }
    assert!(scales[199] < 0.6 && scales[399] == 1.0 && sequence == Sequence::default());
    for pair in scales[199..].windows(2) {
        assert!(pair[1] >= pair[0] && pair[1] / pair[0] < 1.06, "{pair:?}");
    }
}
