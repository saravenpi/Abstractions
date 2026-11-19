//! Tests of the streak of a star in a warp: it is the real apparent motion of that star, from
//! its own distance and bearing and the speed of the ship, and of the edge of sight.

use super::look::{EXPOSURE, SPEEDS};
use super::shader::{SIGHT, STREAK_MOST, sight, streak};
use super::{Drive, Sequence};

/// Game light-years per second of the fastest cruise between the stars of a galaxy.
const CRUISE: f32 = 5.0;

/// Radius in radians of the smallest star the sky draws.
const SMALLEST: f32 = 0.0035;

/// A star dead ahead or dead astern does not streak, a star abeam streaks most, and the streak
/// is the angle the flight moves the star by during one exposure: speed across the line of
/// sight over distance.
#[test]
fn a_star_streaks_by_its_own_apparent_motion() {
    let travel = CRUISE * EXPOSURE;
    assert_eq!(streak(travel, 0.0, 3.0, 1.0), 0.0);
    let abeam = streak(travel, 1.0, 10.0, 1.0);
    assert!(
        (abeam - 0.5 * CRUISE * EXPOSURE / 10.0).abs() < 1e-7,
        "{abeam}"
    );
    assert!((streak(travel, 0.5, 10.0, 1.0) - 0.5 * abeam).abs() < 1e-7);
    assert!((streak(travel, 1.0, 20.0, 1.0) - 0.5 * abeam).abs() < 1e-7);
    assert!((streak(2.0 * travel, 1.0, 10.0, 1.0) - 2.0 * abeam).abs() < 1e-7);
    assert!((streak(travel, 1.0, 10.0, 2.0) - 0.5 * abeam).abs() < 1e-7);
    assert_eq!(streak(0.0, 1.0, 0.01, 1.0), 0.0);
}

/// At cruise a near star streaks long and passes by, a far star stays a point, and no streak
/// is longer than the cap, however near the star.
#[test]
fn near_stars_streak_and_far_stars_stay_points() {
    let travel = CRUISE * EXPOSURE;
    let ranges = [0.01, 0.5, 2.0, 5.0, 10.0, 20.0, 40.0];
    let halves = ranges.map(|range| streak(travel, 1.0, range, 1.0));
    assert!(
        halves.windows(2).all(|pair| pair[1] <= pair[0]),
        "{halves:?}"
    );
    assert_eq!(halves[0], STREAK_MOST);
    assert!(
        halves[2] > 10.0 * SMALLEST,
        "a star 2 light-years abeam streaks"
    );
    assert!(
        halves[6] < SMALLEST,
        "a star at the edge of sight stays a point"
    );
}

/// A far galaxy has a distance too, so the same law holds: it does not streak at cruise speed,
/// and at the speed of a long jump it streaks by its own distance, far less than a star.
#[test]
fn a_far_galaxy_streaks_only_at_jump_speed() {
    let range = 1.0e5;
    let cruise = streak(CRUISE * EXPOSURE, 1.0, range, 1.0);
    let jump = streak(SPEEDS.1 * EXPOSURE, 1.0, range, 1.0);
    assert!(cruise < 2.0e-6, "{cruise}");
    assert!(jump > 5.0e-4 && jump < STREAK_MOST, "{jump}");
    assert_eq!(streak(SPEEDS.1 * EXPOSURE, 1.0, 10.0, 1.0), STREAK_MOST);
}

/// The speed the sky is drawn at is the speed the drive reports, rises with it and never
/// steps: nothing at rest and while the drive spools, the slowest warp once it has engaged,
/// the fastest jump at the top, and the same ratio for the same step of the report.
#[test]
fn the_sky_is_drawn_at_the_speed_of_the_drive() {
    let cruising = |pace: f32| {
        let drive = Drive {
            charge: 1.0,
            speed: pace,
        };
        Sequence::default().advance(drive, 100.0)
    };
    assert_eq!(Sequence::default().speed(), 0.0);
    let spooled = Sequence {
        charge: 0.45,
        ..Sequence::default()
    };
    assert_eq!((spooled.speed(), spooled.streak()), (0.0, 0.0));
    assert!((cruising(0.0).speed() / SPEEDS.0 - 1.0).abs() < 1e-4);
    assert!((cruising(1.0).speed() / SPEEDS.1 - 1.0).abs() < 1e-3);
    let speeds: Vec<f32> = (0..=100)
        .map(|i| cruising(i as f32 / 100.0).speed())
        .collect();
    for pair in speeds.windows(2) {
        let ratio = pair[1] / pair[0];
        assert!(ratio > 1.26 && ratio < 1.28, "{ratio}");
    }
    assert_eq!(cruising(0.6).streak(), cruising(0.6).speed() * EXPOSURE);
}

/// A star keeps all its light up close and fades to nothing over the last stretch before the
/// edge of sight, without a step: a star the ship closes on comes into the sky from nothing.
#[test]
fn a_star_fades_in_at_the_edge_of_sight() {
    assert_eq!((sight(0.0), sight(SIGHT.0 - SIGHT.1)), (1.0, 1.0));
    assert_eq!((sight(SIGHT.0), sight(55.0)), (0.0, 0.0));
    let shares: Vec<f32> = (0..=250).map(|i| sight(37.0 + i as f32 * 0.0125)).collect();
    for pair in shares.windows(2) {
        assert!(pair[1] <= pair[0] && pair[0] - pair[1] < 0.01, "{pair:?}");
    }
    let seconds = SIGHT.1 / CRUISE;
    assert!(
        (seconds - 0.5).abs() < 1e-6,
        "a star fades in over {seconds} s"
    );
}
