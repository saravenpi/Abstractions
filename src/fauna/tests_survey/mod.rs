//! The survey of home planets: for every seed of a range, what the home is like where a new game
//! wakes, printed and counted, and what the rule of the home must give.

mod herd;
mod row;
mod shares;

use row::{Row, measure};

use crate::universe::{COMFORT, EARTH_RADIUS, PlanetClass, SUN_MASS};

/// Seeds the survey looks at.
const SEEDS: u64 = 100;

/// Prints the part of a line of the survey that describes the planet.
fn planet(r: &Row) -> String {
    let (p, s, rec) = (&r.planet, &r.star, &r.planet.record);
    format!(
        "seed {:3} {:?} {:.2}Msun {:.2}AU {} R{:.2} g{:.1} P{:.2} T{:.0}K eq{:.0} pole{:.0} \
         sea{:.2} {:?} day{:.0}h lock{}",
        r.seed,
        s.class,
        s.mass / SUN_MASS,
        r.au(),
        r.zone(),
        p.radius / EARTH_RADIUS,
        p.gravity,
        rec.pressure,
        rec.temperature,
        r.span.0,
        r.span.1,
        rec.ocean,
        rec.class,
        p.day_seconds / 3600.0,
        u8::from(p.locked()),
    )
}

/// Prints one line of the survey.
fn show(r: &Row) {
    let (plants, walkers) = r.species();
    let rung = r.life.as_ref().map(|l| (l.rung, l.chemistry.solvent));
    let land = &r.shares;
    println!(
        "{} life {:?} plants{} walkers{} desert{:.2} grass{:.2} forest{:.2} snow{:.2} spawn{:.0}K {:?} herd{}",
        planet(r),
        rung,
        plants,
        walkers,
        land.desert,
        land.grass,
        land.forest,
        land.snow,
        r.spawn.temperature,
        r.biome,
        u8::from(r.herd),
    );
}

/// Counts the homes by kind and prints the distribution.
fn tally(rows: &[Row]) -> usize {
    let count = |f: &dyn Fn(&Row) -> bool| rows.iter().filter(|r| f(r)).count();
    let class = |c: PlanetClass| count(&|r| r.planet.record.class == c);
    let good = count(&Row::good);
    println!("homes {}", rows.len());
    println!("good (wet, green, walked, 0 to 40 C) {good}");
    println!("desert class {}", class(PlanetClass::Desert));
    println!("ice class {}", class(PlanetClass::Ice));
    println!("locked {}", count(&|r| r.planet.locked()));
    println!("dead {}", count(&|r| r.life.is_none()));
    println!("no liquid water {}", count(&|r| !r.wet()));
    println!("no plants at spawn {}", count(&|r| !r.plants()));
    println!("no walker species {}", count(&|r| r.species().1 == 0));
    println!("no herd within walk {}", count(&|r| !r.herd));
    println!("hot spawn {}", count(&|r| r.spawn.temperature > 313.15));
    println!("cold spawn {}", count(&|r| r.spawn.temperature < 273.15));
    let pull = rows.iter().map(|r| r.planet.gravity);
    let (low, high) = (
        pull.clone().fold(f64::MAX, f64::min),
        pull.fold(0.0, f64::max),
    );
    println!("gravity {low:.2} to {high:.2} m/s2");
    println!(
        "day under 10 h {}",
        count(&|r| r.planet.day_seconds < 36_000.0)
    );
    println!(
        "day over 100 h {}",
        count(&|r| r.planet.day_seconds > 360_000.0)
    );
    good
}

/// Surveys the homes of the first seeds, prints them and the distribution, and asserts that nearly
/// all of them are worlds a new player wakes on with life around: liquid water, plants at the
/// spawn, a walking species and a mean air of 0 to 40 C there, and a herd within a walk of it.
#[test]
fn homes_have_life_to_see() {
    let rows: Vec<Row> = (0..SEEDS).map(measure).collect();
    rows.iter().for_each(show);
    let good = tally(&rows);
    let herds = rows.iter().filter(|r| r.herd).count();
    let (low, high) = COMFORT;
    assert!(
        rows.iter()
            .all(|r| (low..=high).contains(&r.planet.gravity))
    );
    assert!(good * 100 >= 95 * rows.len(), "{good} of {}", rows.len());
    assert!(herds * 10 >= 9 * rows.len(), "herds near on {herds}");
}
