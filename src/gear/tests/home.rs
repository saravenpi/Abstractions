//! The wingsuit on the home planets of new worlds: the air of every one carries a glide, with
//! a third more of it than the glide needs.

use crate::gear::glides;
use crate::gear::tune::SEA_DENSITY;
use crate::universe::{STANDARD_GRAVITY, Universe};
use crate::wind::thickness;

/// Seeds the test looks at.
const SEEDS: u64 = 200;

/// How much thicker than the thinnest air that carries a glide the air of every home is: the
/// thinnest home air, 0.65 of the home air, against the 0.46 the glide needs under the strongest
/// home pull, 1.05 g, is 1.42.
const MARGIN: f32 = 1.35;

/// On the home of each of 200 seeds, at sea level and under its pull, air 1.35 times thinner than
/// its own still carries a hands-off glide, see [`glides`]: the suit glides there with room to
/// spare. The thinnest margin is printed.
#[test]
fn every_home_glides_with_margin() {
    let mut grounded = Vec::new();
    let mut thinnest = f32::MAX;
    for seed in 0..SEEDS {
        let universe = Universe::new(seed);
        let planet = universe.planet(universe.home());
        let (air, gravity) = (thickness(&planet.record) as f32, planet.gravity as f32);
        let needed = (1..=400)
            .map(|step| step as f32 * 0.005)
            .find(|&thin| glides(thin, SEA_DENSITY, gravity))
            .unwrap_or(f32::MAX);
        thinnest = thinnest.min(air / needed);
        if !glides(air / MARGIN, SEA_DENSITY, gravity) {
            let pull = f64::from(gravity) / STANDARD_GRAVITY;
            grounded.push((seed, planet.record.pressure, pull));
        }
    }
    println!("thinnest home air against what the glide needs: {thinnest:.2}");
    println!("homes without room to glide (seed, bar, g): {grounded:?}");
    assert!(grounded.is_empty(), "{grounded:?}");
}
