//! Computer tests: what the numbers of a planet mean for the player, in plain words.

use super::feel::{day_feel, gravity_feel, heat, km, versus};

/// The pull reads from very light to crushing against home.
#[test]
fn the_pull_reads_as_a_feel() {
    let pulls = [0.1, 0.3, 0.79, 0.8, 1.0, 1.25, 1.99, 2.0, 5.0];
    let said = pulls.map(gravity_feel);
    assert_eq!(said[0], "Very light: huge jumps");
    assert_eq!([said[1], said[2]], ["Light", "Light"]);
    assert_eq!([said[3], said[4]], ["Like home", "Like home"]);
    assert_eq!(
        [said[5], said[6]],
        ["Heavy: short jumps", "Heavy: short jumps"]
    );
    assert_eq!([said[7], said[8]], ["Crushing", "Crushing"]);
}

/// The temperature and the day read as one plain word each.
#[test]
fn the_heat_and_the_day_read_as_words() {
    let kelvin = [153.15, 175.0, 272.0, 274.0, 300.0, 330.0, 380.0, 500.0];
    let expected = [
        "Freezing",
        "Cold",
        "Cold",
        "Mild",
        "Mild",
        "Hot",
        "Scorching",
        "Scorching",
    ];
    assert_eq!(kelvin.map(heat), expected);
    assert_eq!(day_feel(0.2), "Short days: the sun moves fast");
    assert_eq!(day_feel(1.0), "Days like home");
    assert_eq!(day_feel(3.0), "Long days: nights last for hours");
}

/// A ratio reads against home with one decimal, two under a tenth and none past a hundred, and a
/// length reads in kilometres with commas.
#[test]
fn ratios_and_lengths_read_with_their_unit() {
    assert_eq!(versus(1.2, 1.0), "1.2 x home");
    assert_eq!(versus(1.0, 1.0), "1.0 x home");
    assert_eq!(versus(0.04, 1.0), "0.04 x home");
    assert_eq!(versus(1500.0, 1.0), "1500 x home");
    assert_eq!(versus(1.0, 0.0), "");
    assert_eq!(km(3_390_000.0), "3,390 km");
    assert_eq!(km(6000.0), "6 km");
    assert_eq!(km(69_911_000.0), "69,911 km");
    assert_eq!(km(1.0e9), "1,000,000 km");
    assert_eq!(km(0.0), "0 km");
}
