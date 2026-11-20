//! The processor time of the calling thread, for the tests that time work: a machine busy with other tests
//! does not change what they measure.

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
pub fn busy() -> f64 {
    let mut time = Timespec::default();
    let failed = unsafe { clock_gettime(THREAD_CLOCK, &mut time) };
    assert_eq!(failed, 0);
    time.seconds as f64 + time.nanos as f64 * 1.0e-9
}
