//! The test rig of the signs: the meadow life of the herd fixtures, its ground, watches over it, and
//! the moments the tests look at.

use bevy::math::DVec3;

use super::{Sign, near};
use crate::life::herd::fixtures::{light, sample};
use crate::life::herd::{Ground, Herd, HerdState, Search, Watch};
use crate::life::hunt::{Carcass, Hunt};
use crate::life::{Life, hunt};

/// Radius of the test planet in metres.
pub const RADIUS: f64 = 6000.0;

/// Returns a point on land of the test planet.
pub fn land() -> DVec3 {
    DVec3::new(1.0, 0.3, 0.2).normalize()
}

/// Returns the search of `radius` metres around `at` at `time` on the test planet.
pub fn search(at: DVec3, radius: f64, time: f64) -> Search {
    Search {
        at,
        radius,
        planet: RADIUS,
        time,
    }
}

/// Calls `f` with the ground of the test planet under full daylight.
pub fn on<T>(f: impl FnOnce(&Ground) -> T) -> T {
    let site = |d| sample(d, 1.0);
    f(&Ground {
        sample: &site,
        light: &light,
    })
}

/// Returns the signs of `life` within 60 m of `at` at `time`, from a watch rolled there and then.
pub fn signs(life: &Life, at: DVec3, time: f64) -> Vec<Sign> {
    on(|g| {
        let mut watch = Watch::new(life, g, &search(at, 400.0, time));
        let mut out = Vec::new();
        near(life, g, &mut watch, &search(at, 60.0, time), &mut out);
        out
    })
}

/// Returns the first herd of `life` around [`land`] doing what `wanted` asks, at one of a few times,
/// with that time.
pub fn find(life: &Life, wanted: impl Fn(&Herd) -> bool) -> Option<(Herd, f64)> {
    let times = (0..40).map(|k| 900.0 + 173.0 * f64::from(k));
    on(|g| {
        times.into_iter().find_map(|t| {
            let herds = crate::life::herd::near(life, g, &search(land(), 2500.0, t));
            herds.into_iter().find(|h| wanted(h)).map(|h| (h, t))
        })
    })
}

/// Returns true when `herd` walks between waypoints.
pub fn travels(herd: &Herd) -> bool {
    herd.state == HerdState::Travel && herd.speed > 0.5
}

/// Returns the first carcass of a kill around [`land`] over two hours.
pub fn carcass(life: &Life) -> Option<Carcass> {
    on(|g| {
        (0..48).find_map(|k| {
            let at = search(land(), 2500.0, 3600.0 + 150.0 * f64::from(k));
            hunt::carcasses_near(life, g, &at).into_iter().next()
        })
    })
}

/// Returns the first hunt around [`land`] over two hours with a chase of more than 3 s.
pub fn chase(life: &Life) -> Option<Hunt> {
    on(|g| {
        (0..24)
            .flat_map(|k| {
                hunt::hunts_near(
                    life,
                    g,
                    &search(land(), 2500.0, 3600.0 + 300.0 * f64::from(k)),
                )
            })
            .find(|h| h.chase > 3.0)
    })
}
