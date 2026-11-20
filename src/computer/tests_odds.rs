//! Computer tests: the line on life of the card, the chance before a landing and what lives
//! there after, and what working it out costs.

use std::time::Instant;

use super::detail::Row;
use super::odds::{Odds, lives, percent, row, world};
use crate::life::{Rung, odds};
use crate::ui::Icon;
use crate::universe::Universe;

/// A chance reads in whole percent, and only a sure thing reads 0 or 100.
#[test]
fn a_chance_reads_in_whole_percent() {
    let said = [0.0, 0.004, 0.42, 0.716, 0.996, 1.0].map(percent);
    assert_eq!(said, ["0 %", "1 %", "42 %", "72 %", "99 %", "100 %"]);
}

/// Each rung reads in plain words, and a dead world says none.
#[test]
fn what_lives_reads_in_plain_words() {
    let rungs = [
        None,
        Some(Rung::Mats),
        Some(Rung::Soft),
        Some(Rung::Shelled),
    ];
    let higher = [Some(Rung::Skeletal), Some(Rung::Old)];
    let said: Vec<&str> = rungs.into_iter().chain(higher).map(lives).collect();
    let plain = ["none", "microbial mats", "soft-bodied life", "shelled life"];
    assert_eq!(said[..4], plain);
    assert_eq!(said[4..], ["animals with skeletons", "ancient life"]);
}

/// Before a landing the card gives the chance of the life model, after one what the model put
/// there, the same rung the planet is entered with. Prints both for the home planet.
#[test]
fn the_line_follows_the_landing() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let star = universe.system(home.id.star).star;
    let chance = odds::chance(&world(&universe, &home));
    let rung = crate::life::of(&home, &star).map(|life| life.rung);
    let before = row(&universe, &home, false);
    let after = row(&universe, &home, true);
    println!("home: {before:?}, then {after:?}");
    let life = |label, value: String| Row::Stat {
        icon: Icon::Life,
        label,
        value,
        note: String::new(),
    };
    assert_eq!(before, life("Chance of life", percent(chance)));
    assert_eq!(after, life("Life", lives(rung).to_string()));
    assert!(chance > 0.5);
}

/// The line is worked out once per body and landing: the card asks for it twice a second while
/// a row stays selected, and those asks cost nothing. One working out stays far under a frame.
#[test]
fn the_line_is_kept_while_the_body_stays_selected() {
    let universe = Universe::new(7);
    let home = universe.planet(universe.home());
    let planets = universe.system(home.id.star).planets;
    let other = planets
        .into_iter()
        .find(|planet| planet.id != home.id)
        .unwrap();
    let mut kept = Odds::default();
    let start = Instant::now();
    let first = kept.rows(&universe, &home, false);
    let once = start.elapsed();
    let start = Instant::now();
    assert!((0..1000).all(|_| kept.rows(&universe, &home, false) == first));
    let again = start.elapsed();
    println!("one line in {once:?}, a thousand kept ones in {again:?}");
    assert_eq!(kept.worked, 1);
    kept.rows(&universe, &other, false);
    kept.rows(&universe, &other, true);
    assert_eq!(kept.worked, 3);
    assert!(
        once.as_millis() < 20 && again.as_millis() < 5,
        "{once:?} {again:?}"
    );
}
