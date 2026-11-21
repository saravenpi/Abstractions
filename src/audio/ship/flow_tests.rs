//! Tests of the air on the airframe against the real flow: it rises with the dynamic pressure, a
//! high angle of attack is louder and broader, a sideslip pans it, a step of the stick is heard
//! within 0.2 seconds, steady flight with a restless stick is steady, and a hard roll whooshes once.

use super::airframe::{Airframe, Flow, balance};
use super::airframe_tests::{DT, RATE, loudness};
use super::whirl::Whirl;
use crate::audio::dsp::filter::Svf;
use crate::audio::dsp::rand::Rand;

/// A cruise at 120 metres per second at sea level at home: about 8.6 kPa.
const CRUISE: Flow = Flow {
    speed: 120.0,
    pressure: 8_640.0,
    ..Flow::STILL
};

/// Returns how much of a second of `flow` lies far from the centre of its wind band: the level
/// at a quarter and at four times the centre, over the level at the centre.
fn breadth(flow: Flow) -> f32 {
    let mut frame = Airframe::new(RATE, 3);
    frame.tune(&flow, DT);
    let centre = 220.0 + 5.0 * flow.speed;
    let mut bands = [centre * 0.25, centre, centre * 4.0].map(|hz| {
        let mut band = Svf::default();
        band.tune(hz, 4.0, RATE);
        (band, 0.0_f32)
    });
    for i in 0..(1.5 * RATE) as usize {
        let x = frame.frame();
        for (band, sum) in &mut bands {
            let y = band.band(x);
            *sum += if i as f32 > 0.5 * RATE { y * y } else { 0.0 };
        }
    }
    (bands[0].1 + bands[2].1) / bands[1].1.max(1.0e-12)
}

/// The wind grows with the dynamic pressure: 10 dB for each tenfold pressure.
#[test]
fn the_air_rises_with_the_dynamic_pressure() {
    let at = |pressure: f32| loudness(Flow { pressure, ..CRUISE });
    let (low, high) = (at(864.0), at(8_640.0));
    let rise = 20.0 * (high / low).log10();
    assert!(high > low && (rise - 10.0).abs() < 2.0, "{rise} dB");
}

/// At the same dynamic pressure a 15 degree angle of attack tears the flow: louder and broader.
#[test]
fn a_high_angle_of_attack_is_louder_and_broader() {
    let pulled = Flow {
        attack: 15.0_f32.to_radians(),
        ..CRUISE
    };
    assert!(loudness(pulled) > 1.2 * loudness(CRUISE));
    assert!(breadth(pulled) > 1.2 * breadth(CRUISE));
}

/// A sideslip from the right makes the right side louder, from the left the left, and straight
/// flight is even.
#[test]
fn a_sideslip_pans_the_air() {
    let [left, right] = balance(0.3);
    assert!(right > 1.2 * left);
    let [left, right] = balance(-0.3);
    assert!(left > 1.2 * right);
    assert_eq!(balance(0.0), [1.0, 1.0]);
}

/// A step of the stick into a hard pull is heard within 0.2 seconds: against the same airframe
/// that jumped at once, it has a fraction of the change after 25 milliseconds and most of it by
/// 0.2 seconds.
#[test]
fn a_step_of_the_stick_is_heard_within_a_fifth_of_a_second() {
    let mut eased = Airframe::new(RATE, 5);
    eased.tune(&CRUISE, DT);
    for _ in 0..(0.5 * RATE) as usize {
        eased.frame();
    }
    let pull = Flow {
        attack: 0.5,
        load: 5.0,
        ..CRUISE
    };
    eased.tune(&pull, DT);
    let mut jumped = eased;
    jumped.snap();
    let window = |frame: &mut Airframe| {
        let samples: Vec<f32> = (0..(0.05 * RATE) as usize).map(|_| frame.frame()).collect();
        samples.iter().map(|x| x * x).sum::<f32>().sqrt()
    };
    let share: Vec<f32> = (0..4)
        .map(|_| window(&mut eased) / window(&mut jumped))
        .collect();
    assert!(share[0] < 0.6 && share[3] > 0.75, "{share:?}");
    assert!(share.windows(2).all(|pair| pair[1] > pair[0]), "{share:?}");
}

/// A minute of steady cruise in calm air, with the small corrections of a restless stick on the
/// angle, the sideslip, the rates and the load, is steady: no half second differs from another by
/// more than the grain of the noise itself.
#[test]
fn steady_flight_is_steady() {
    let mut frame = Airframe::new(RATE, 11);
    let mut rand = Rand::new(5, 0);
    let mut windows = Vec::new();
    let (block, window) = ((DT * RATE) as usize, (0.5 * RATE) as usize);
    let mut sum = 0.0_f32;
    for n in 0..(60.0 / DT) as usize {
        frame.tune(
            &Flow {
                attack: rand.range(0.02, 0.06),
                slip: rand.range(-0.02, 0.02),
                turn: rand.range(0.0, 0.5),
                load: rand.range(0.8, 1.4),
                ..CRUISE
            },
            DT,
        );
        for i in 0..block {
            sum += frame.frame().powi(2);
            if (n * block + i + 1) % window == 0 {
                windows.push(sum.sqrt());
                sum = 0.0;
            }
        }
    }
    let settled = &windows[2..];
    let (low, high) = settled
        .iter()
        .fold((f32::MAX, 0.0_f32), |(l, h), x| (l.min(*x), h.max(*x)));
    let depth = (high - low) / (high + low);
    assert!(depth < 0.15, "the bed swings by {depth}");
}

/// A single hard roll whooshes once, however the rate wavers about the onset, and the small rates
/// of steady flight never do.
#[test]
fn a_hard_roll_whooshes_once() {
    let whooshes = |rate: fn(f32) -> f32| {
        let mut whirl = Whirl::default();
        let mut rand = Rand::new(9, 0);
        let (mut count, mut before) = (0, 0.0);
        for n in 0..(4.0 / DT) as usize {
            whirl.tune(rate(n as f32 * DT) + rand.range(-0.4, 0.4), DT);
            count += usize::from(whirl.level() > before + 0.5);
            before = whirl.level();
        }
        count
    };
    let roll = |t: f32| if (0.5..2.0).contains(&t) { 2.0 } else { 0.0 };
    assert_eq!(whooshes(roll), 1);
    assert_eq!(whooshes(|_| 0.5), 0);
}
