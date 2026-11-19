//! Cost of the assembly: the life of a planet must be built in under 5 ms at opt-level 1, measured
//! with the processor time of the test thread, which the load of the other tests does not inflate.

use std::hint::black_box;

use super::*;
use crate::universe::Universe;

/// A point in time as the C library gives it.
#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default)]
struct Timespec {
    /// Whole seconds.
    seconds: i64,
    /// Nanoseconds past the whole seconds.
    nanos: i64,
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    /// Reads the clock `clock` into `time`.
    fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
}

/// The clock that counts the processor time of the calling thread on Linux.
#[cfg(target_os = "linux")]
const THREAD_CLOCK: i32 = 3;

/// Returns the seconds of processor time the calling thread has used.
#[cfg(target_os = "linux")]
fn busy() -> f64 {
    let mut time = Timespec::default();
    let failed = unsafe { clock_gettime(THREAD_CLOCK, &mut time) };
    assert_eq!(failed, 0);
    time.seconds as f64 + time.nanos as f64 * 1.0e-9
}

/// Assembling the life of Kodya costs under 5 ms of processor time.
#[cfg(target_os = "linux")]
#[test]
fn assemble_costs_under_5_ms() {
    let universe = Universe::new(5);
    let home = universe.home();
    let (planet, star) = (universe.planet(home), universe.system(home.star).star);
    let rounds = 40;
    let start = busy();
    for _ in 0..rounds {
        black_box(assemble(black_box(&planet), black_box(&star)));
    }
    let ms = (busy() - start) * 1.0e3 / f64::from(rounds);
    println!("web::assemble costs {ms:.3} ms");
    assert!(ms < 5.0, "web::assemble costs {ms:.3} ms");
}
