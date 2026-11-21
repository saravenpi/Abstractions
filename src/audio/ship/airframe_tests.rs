//! Tests of the airframe: the roar of an entry follows the heat of the nose, and still air at
//! the edge of space makes no sound on the airframe at all.

use super::airframe::{Airframe, Flow};

/// Sample rate of the tests, in hertz.
pub(super) const RATE: f32 = 48_000.0;

/// Returns the root mean square of a second of the airframe in `flow`, after half a second to
/// settle.
pub(super) fn loudness(flow: Flow) -> f32 {
    let mut frame = Airframe::new(RATE, 7);
    frame.tune(&flow);
    let settle = (RATE * 0.5) as usize;
    let samples: Vec<f32> = (0..settle + RATE as usize).map(|_| frame.frame()).collect();
    let tail = &samples[settle..];
    (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt()
}

/// Thin air with no heat, as at the top of a climb, is silent; the same air with the heat of an
/// entry roars, and harder for more heat.
#[test]
fn the_roar_follows_the_heat_and_thin_air_is_silent() {
    let thin = Flow {
        speed: 7000.0,
        ..Flow::STILL
    };
    assert_eq!(loudness(thin), 0.0);
    let warm = loudness(Flow { roar: 0.3, ..thin });
    let hot = loudness(Flow { roar: 1.0, ..thin });
    assert!(warm > 0.0 && hot > warm * 2.0, "{warm} {hot}");
}
