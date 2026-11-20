//! Tests of the fade between two maps of the galaxy light.

use super::fade::{Fade, mix};
use super::tune::FADE;

/// The mix is the first map at 0, the second at 1, the middle at one half, and never leaves
/// the two.
#[test]
fn the_mix_runs_from_one_map_to_the_other() {
    let (from, to) = ([0u8, 100, 255, 40], [200u8, 100, 5, 40]);
    let mut out = [0u8; 4];
    mix(&from, &to, 0.0, &mut out);
    assert_eq!(out, from);
    mix(&from, &to, 1.0, &mut out);
    assert_eq!(out, to);
    mix(&from, &to, 0.5, &mut out);
    assert_eq!(out, [100, 100, 130, 40]);
    for step in 0..=20 {
        mix(&from, &to, step as f32 / 20.0, &mut out);
        assert!(out[0] <= 200 && out[2] >= 5);
    }
}

/// The first map shows whole at once, a later one fades in over [`FADE`] seconds from what was
/// on show, also when it lands in the middle of the fade before, and then the fade is over.
#[test]
fn a_new_map_fades_in_from_what_is_on_show() {
    let mut fade = Fade::default();
    assert!(fade.step(0.0).is_none());
    fade.begin(vec![100; 8], 10.0);
    assert_eq!(fade.step(10.0).map(<[u8]>::to_vec), Some(vec![100; 8]));
    fade.begin(vec![200; 8], 20.0);
    assert_eq!(fade.step(20.0).map(<[u8]>::to_vec), Some(vec![100; 8]));
    assert_eq!(
        fade.step(20.0 + 0.5 * FADE).map(<[u8]>::to_vec),
        Some(vec![150; 8])
    );
    fade.begin(vec![50; 8], 20.0 + 0.5 * FADE);
    assert_eq!(
        fade.step(20.0 + 0.5 * FADE).map(<[u8]>::to_vec),
        Some(vec![150; 8])
    );
    assert_eq!(
        fade.step(20.0 + FADE).map(<[u8]>::to_vec),
        Some(vec![100; 8])
    );
    assert_eq!(
        fade.step(20.0 + 1.5 * FADE).map(<[u8]>::to_vec),
        Some(vec![50; 8])
    );
    assert!(fade.step(30.0 + 2.0 * FADE).is_none());
}
