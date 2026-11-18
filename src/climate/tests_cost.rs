//! Cost of the climate field: one call must stay under 150 ns at opt-level 1, measured with the
//! processor time of the test thread, which the load of the other tests does not inflate.

use std::hint::black_box;

use bevy::math::Vec2;

use super::tests::{kodya, spot};
use super::*;

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

/// One call of `at` costs under 150 ns of processor time, over spots spread on the whole globe.
#[cfg(target_os = "linux")]
#[test]
fn one_sample_costs_under_150_ns() {
    let home = kodya();
    let spots: Vec<Spot> = (0..1024)
        .map(|i| Spot {
            altitude: (i % 9) as f32 * 20.0,
            inland: (i % 5) as f32 * 0.3 - 0.4,
            slope: Vec2::new(0.1, -0.05),
            ..spot(
                (f64::from(i) * 0.1757).sin().asin().to_degrees(),
                f64::from(i) * 137.508,
            )
        })
        .collect();
    let rounds = 200;
    let start = busy();
    for round in 0..rounds {
        for s in &spots {
            black_box(at(
                black_box(&home),
                black_box(s),
                round as f32 / rounds as f32,
            ));
        }
    }
    let ns = (busy() - start) * 1.0e9 / (rounds * spots.len()) as f64;
    println!("climate::at costs {ns:.1} ns");
    assert!(ns < 150.0, "climate::at costs {ns:.1} ns");
}
