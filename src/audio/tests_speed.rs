//! Tests of the sounds of speed: the rumble of buffeting and the pitch of the rush.

use super::suit::{Suit, SuitCtl};

/// Rate the voices are rendered at in these tests.
const RATE: f32 = 48_000.0;

/// Returns the level and the brightness (how fast the signal moves against its level) of one
/// second of the gear voices at `ctl`.
fn heard(ctl: SuitCtl) -> (f32, f32) {
    let mut suit = Suit::new(RATE);
    suit.tune(&ctl);
    let samples: Vec<f32> = (0..RATE as usize).map(|_| suit.frame(&ctl)).collect();
    let level = (samples.iter().map(|x| x * x).sum::<f32>() / samples.len() as f32).sqrt();
    let moving = samples.windows(2).map(|pair| (pair[1] - pair[0]).abs());
    (
        level,
        moving.sum::<f32>() / samples.len() as f32 / level.max(1.0e-9),
    )
}

/// Returns the controls of the gear at rest.
fn calm() -> SuitCtl {
    SuitCtl {
        rush: 0.0,
        speed: 0.0,
        buffet: 0.0,
        boost: 0.0,
        flutter: 0.0,
    }
}

/// The body makes no sound without buffeting, and a low rumble with it.
#[test]
fn buffeting_rumbles() {
    assert!(heard(calm()).0 < 1.0e-6);
    let (level, brightness) = heard(SuitCtl {
        buffet: 1.0,
        ..calm()
    });
    assert!(level > 0.005, "{level}");
    assert!(brightness < 0.5, "{brightness}");
}

/// The same rush is higher in pitch when the air moves faster, and as loud as its level says.
#[test]
fn the_rush_rises_in_pitch_with_speed() {
    let rush = |speed| {
        heard(SuitCtl {
            rush: 0.5,
            speed,
            ..calm()
        })
    };
    let (slow, fast) = (rush(0.1), rush(0.9));
    assert!(fast.1 > 1.5 * slow.1, "{slow:?} {fast:?}");
}

/// Thin air makes the same speed quieter.
#[test]
fn thin_air_is_quieter() {
    let rush = |rush| {
        heard(SuitCtl {
            rush,
            speed: 0.8,
            ..calm()
        })
        .0
    };
    assert!(rush(0.3) < rush(0.9));
}
