//! Tests of the fade of the blades with the distance: no blade pops in or out, the bands nest, and the shader
//! draws with the numbers the tiles are planned with.

use super::fade::{grown, kept, ranks, reach};
use super::tune::{BANDS, END, FADE, FULL, SHORT, SOFT};

/// Most a blade may change in height, as a share of its full height, for one centimetre of distance.
const STEP: f32 = 0.02;

/// The shader of the blades.
const SHADER: &str = include_str!("blades.wesl");

/// Returns the value of the `f32` constant `name` of the blade shader.
fn constant(name: &str) -> f32 {
    let line = format!("const {name}: f32 = ");
    let start = SHADER
        .find(&line)
        .unwrap_or_else(|| panic!("no {name} in blades.wesl"))
        + line.len();
    let rest = &SHADER[start..];
    rest[..rest.find(';').expect("a semicolon")]
        .parse()
        .expect("a number")
}

/// Every blade, of any rank on any ground, grows and shrinks with the distance by less than [`STEP`] of its height
/// for each centimetre, from the camera to past the last blade, and none stands past [`END`].
#[test]
fn the_fade_is_continuous() {
    let blades = (0..200).map(|n| (n as f32 + 0.5) / 200.0);
    for rank in blades {
        for thick in [0.25, 0.5, 0.75, 1.0] {
            let heights: Vec<f32> = (0..6000)
                .map(|cm| grown(rank, cm as f32 * 0.01, thick))
                .collect();
            for (cm, pair) in heights.windows(2).enumerate() {
                let step = (pair[1] - pair[0]).abs();
                assert!(
                    step <= STEP,
                    "rank {rank} thick {thick} jumps {step} at {} m",
                    cm as f32 * 0.01
                );
            }
            assert_eq!(grown(rank, END, thick), 0.0);
        }
    }
}

/// The share of the blades kept never rises with the distance, stands whole within [`FULL`], and every band
/// reaches no farther than the band before it, the farthest to [`END`].
#[test]
fn the_bands_nest() {
    assert_eq!(kept(FULL * 0.5), 1.0);
    let shares: Vec<f32> = (0..500).map(|n| kept(n as f32 * 0.1)).collect();
    assert!(shares.windows(2).all(|p| p[1] <= p[0]));
    assert_eq!(reach(0), END);
    for band in 1..BANDS.len() {
        assert_eq!(ranks(band).start, ranks(band - 1).end);
        assert!(reach(band) <= reach(band - 1));
        let low = ranks(band).start;
        assert!(
            grown(low, reach(band) + 0.01, 1.0) == 0.0,
            "band {band} draws past its reach"
        );
    }
}

/// The shader fades the blades with the numbers of `tune.rs`.
#[test]
fn the_shader_fades_as_the_tiles_plan() {
    for (name, value) in [
        ("FULL", FULL),
        ("END", END),
        ("FADE", FADE),
        ("SOFT", SOFT),
        ("SHORT", SHORT),
    ] {
        assert_eq!(constant(name), value, "{name} of blades.wesl");
    }
}
