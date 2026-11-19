//! Cost of the assembly: the life of a planet must be built in under 5 ms at opt-level 1, measured
//! with the processor time of the test thread, which the load of the other tests does not inflate.

use std::hint::black_box;

use super::*;
use crate::universe::Universe;

/// A point in time as the C library gives it.
#[repr(C)]
#[derive(Default)]
struct Timespec {
    /// Whole seconds.
    seconds: i64,
    /// Nanoseconds past the whole seconds.
    nanos: i64,
}

unsafe extern "C" {
    /// Reads the clock `clock` into `time`.
    fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
}

/// The clock that counts the processor time of the calling thread on Linux.
#[cfg(target_os = "linux")]
const THREAD_CLOCK: i32 = 3;

/// The clock that counts the processor time of the calling thread on macOS.
#[cfg(target_os = "macos")]
const THREAD_CLOCK: i32 = 16;

/// Returns the seconds of processor time the calling thread has used.
fn busy() -> f64 {
    let mut time = Timespec::default();
    let failed = unsafe { clock_gettime(THREAD_CLOCK, &mut time) };
    assert_eq!(failed, 0);
    time.seconds as f64 + time.nanos as f64 * 1.0e-9
}

/// Returns the milliseconds of processor time one call of `build` on `planet` around `star` costs.
fn cost(
    planet: &PlanetSpec,
    star: &StarSpec,
    build: fn(&PlanetSpec, &StarSpec) -> Option<Life>,
) -> f64 {
    let rounds = 40;
    let start = busy();
    for _ in 0..rounds {
        black_box(build(black_box(planet), black_box(star)));
    }
    (busy() - start) * 1.0e3 / f64::from(rounds)
}

/// Assembling the life of Kodya costs under 5 ms of processor time, through the web and through
/// `life::of`, which runs when the planet is entered.
#[test]
fn assemble_costs_under_5_ms() {
    let universe = Universe::new(5);
    let home = universe.home();
    let (planet, star) = (universe.planet(home), universe.system(home.star).star);
    let web = cost(&planet, &star, assemble);
    let of = cost(&planet, &star, crate::life::of);
    println!("web::assemble costs {web:.3} ms, life::of {of:.3} ms");
    assert!(web < 5.0, "web::assemble costs {web:.3} ms");
    assert!(of < 5.0, "life::of costs {of:.3} ms");
}

/// The richest web around the home sector, the most species, also costs under 5 ms.
#[test]
fn richest_web_costs_under_5_ms() {
    let bodies = crate::life::odds::tests::systems(40)
        .into_iter()
        .flat_map(|(star, bodies)| bodies.into_iter().map(move |p| (star.clone(), p)));
    let richest = bodies
        .filter_map(|(star, p)| assemble(&p, &star).map(|life| (life.species.len(), star, p)))
        .max_by_key(|(count, _, _)| *count);
    let (count, star, planet) = richest.expect("a living planet near home");
    let ms = cost(&planet, &star, crate::life::of);
    println!("life::of on {count} species costs {ms:.3} ms");
    assert!(ms < 5.0, "life::of on {count} species costs {ms:.3} ms");
}
