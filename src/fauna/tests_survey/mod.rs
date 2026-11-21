//! The survey of home planets: for every seed of a range, what the home is like where a new game
//! wakes, printed and counted, and what the rule of the home must give.

mod glide;
mod herd;
mod row;
mod shares;

use row::{Row, measure};

use crate::universe::{
    COMFORT, EARTH_RADIUS, GLIDE_AIR, LIT, MILD, PlanetClass, STANDARD_GRAVITY, SUN_MASS, dense,
};

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
        "{} life {:?} plants{} walkers{} desert{:.2} grass{:.2} forest{:.2} snow{:.2} spawn{:.0}K {:?} herd{} \
         star{:.0}deg glide{:.1}m/s sink{:.1}m/s",
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
        r.noon.min(1.0).asin().to_degrees(),
        r.glide.forward,
        r.glide.sink,
    );
}

/// Prints the distribution of the home pressures and the glides, and checks that every home has
/// the air [`GLIDE_AIR`] asks for and that the wingsuit glides over its base inside the flyable
/// envelope, see [`glide::Glide::flyable`]. Prints the seeds that do not.
fn air(rows: &[Row]) {
    let mut pressures: Vec<f64> = rows.iter().map(|r| r.planet.record.pressure).collect();
    pressures.sort_by(f64::total_cmp);
    let at = |share: f64| pressures[((pressures.len() - 1) as f64 * share).round() as usize];
    println!(
        "pressure min {:.2} quartile {:.2} median {:.2} quartile {:.2} max {:.2} bar",
        at(0.0),
        at(0.25),
        at(0.5),
        at(0.75),
        at(1.0)
    );
    let worst = rows.iter().map(|r| r.glide.sink).fold(0.0, f32::max);
    println!("fastest glide sink {worst:.1} m/s");
    let seeds = |f: &dyn Fn(&Row) -> bool| -> Vec<u64> {
        rows.iter().filter(|r| f(r)).map(|r| r.seed).collect()
    };
    let thin =
        seeds(&|r| dense(&r.planet.record) < GLIDE_AIR * r.planet.gravity / STANDARD_GRAVITY);
    let grounded = seeds(&|r| !r.glide.flyable());
    println!("air too thin {thin:?}");
    println!("glide out of the envelope {grounded:?}");
    assert!(thin.is_empty(), "air too thin on {thin:?}");
    assert!(grounded.is_empty(), "no glide on {grounded:?}");
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

/// Checks that every home wakes in daylight: over a day the star climbs to [`LIT`] at the spawn, less the
/// hundredth of sine a spawn may lose to the shore it was looked for around. Checks that nearly every home
/// wakes in mild mean air, within the 3 K the slow warp of the latitudes moves it by. Prints the seeds that
/// do not, with the lowest star of all.
fn fair(rows: &[Row]) {
    let lit = LIT.to_radians().sin() - 0.01;
    let mild = MILD.0 as f32 - 3.0..=MILD.1 as f32 + 3.0;
    let seeds = |f: &dyn Fn(&Row) -> bool| -> Vec<u64> {
        let kept = rows.iter().filter(|r| f(r));
        kept.map(|r| r.seed).collect()
    };
    let dark = seeds(&|r| r.noon < lit);
    let harsh = seeds(&|r| !mild.contains(&r.spawn.temperature));
    let lowest = rows.iter().map(|r| r.noon).fold(f64::MAX, f64::min);
    println!("lowest star {:.1} degrees", lowest.asin().to_degrees());
    println!("spawn the star stays low over {dark:?}");
    println!("spawn out of the mild air {harsh:?}");
    assert!(dark.is_empty(), "the star stays low over {dark:?}");
    assert!(harsh.len() * 20 <= rows.len(), "not mild: {harsh:?}");
}

/// Surveys the homes of the first seeds, prints them and the distribution, and asserts that nearly
/// all of them are worlds a new player wakes on with life around: liquid water, plants at the
/// spawn, a walking species and a mean air of 0 to 40 C there, and a herd within a walk of it. Every
/// one of them wakes in daylight, see [`fair`], and glides in its air, see [`air`].
#[test]
fn homes_have_life_to_see() {
    let rows: Vec<Row> = (0..SEEDS).map(measure).collect();
    rows.iter().for_each(show);
    fair(&rows);
    air(&rows);
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
