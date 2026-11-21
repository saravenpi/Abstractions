//! Test of the cost of the bands of blades: wherever the camera stands, the vertex shader takes up no more blades
//! than it did for the rings of 0.11.0.

use bevy::prelude::*;

use super::fade::ranks;
use super::tiles::{Eye, within};
use super::tune::{BANDS, DENSE, TILE};

/// Returns how many blades the vertex shader takes up around a camera at `at` on the ground: the rings of
/// 0.11.0, dense blades in tiles whose middle lies within 23 m and eight a square metre in those within 48 m,
/// and the bands of now, each in the tiles whose nearest point lies within the distance it is built at.
fn taken_up(at: Vec2) -> (f32, f32) {
    let span = 8;
    let keys = (-span..=span).flat_map(|x| (-span..=span).map(move |y| IVec2::new(x, y)));
    let area = TILE * TILE;
    let (mut before, mut now) = (0.0, 0.0);
    for key in keys {
        let middle = ((key.as_vec2() + 0.5) * TILE).distance(at);
        before += match middle {
            d if d < 23.0 => area * DENSE,
            d if d < 48.0 => area * 8.0,
            _ => 0.0,
        };
        let away = Eye::new(at, 1.7).away(key);
        let held = (0..BANDS.len()).filter(|band| away < within(*band));
        now += held
            .map(|band| (ranks(band).end - ranks(band).start) * area * DENSE)
            .sum::<f32>();
    }
    (before, now)
}

/// Over the places the camera may stand in its tile the bands hold no more blades than the rings of 0.11.0, at
/// most or on average: the vertex shader takes up no more of them.
#[test]
fn the_bands_cost_no_more_than_the_rings() {
    let spots = (0..8).flat_map(|x| (0..4).map(move |y| Vec2::new(x as f32, y as f32 * 2.0)));
    let counts: Vec<(f32, f32)> = spots.map(taken_up).collect();
    let mean = |n: fn(&(f32, f32)) -> f32| counts.iter().map(n).sum::<f32>() / counts.len() as f32;
    let most = |n: fn(&(f32, f32)) -> f32| counts.iter().map(n).fold(0.0, f32::max);
    let (before, now) = (
        (most(|c| c.0), mean(|c| c.0)),
        (most(|c| c.1), mean(|c| c.1)),
    );
    println!("blades taken up, most and mean: {now:?} now, {before:?} in the rings of 0.11.0");
    assert!(
        now.0 <= before.0 && now.1 <= before.1,
        "{now:?} blades in the bands, {before:?} in the rings"
    );
}
