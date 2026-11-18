//! The app a flyover runs in: the streaming chain on one thread, with the processor time of
//! every step noted, so a busy machine does not change what is measured.

use std::time::Duration;

use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::STEP;
use crate::world::Planet;
use crate::world::chunks::{self, Chunks};
use crate::world::props::PropAssets;

/// The steps of the streaming chain that are timed, in the order they run.
pub const STEPS: [&str; 8] = [
    "face", "far", "grow", "stream", "collect", "refine", "show", "place",
];

/// A point in time as the C library gives it.
#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default)]
struct Timespec {
    seconds: i64,
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
pub fn busy() -> f64 {
    let mut time = Timespec::default();
    let failed = unsafe { clock_gettime(THREAD_CLOCK, &mut time) };
    assert_eq!(failed, 0);
    time.seconds as f64 + time.nanos as f64 * 1.0e-9
}

/// Returns the seconds since the first call: where threads have no clock, the wall has one.
#[cfg(not(target_os = "linux"))]
pub fn busy() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(std::time::Instant::now);
    start.elapsed().as_secs_f64()
}

/// How many times longer than a 60 Hz frame the wall clock gives each frame of a flyover. The
/// simulated time does not change: a test that runs beside others uses it so that its worker
/// threads, which every test of the run shares, are not what it measures.
#[derive(Resource, Debug, Clone, Copy)]
pub struct Slow(pub f64);

/// Processor milliseconds of each step in the last frame.
#[derive(Resource, Default)]
pub struct Laps {
    last: f64,
    /// The cost of each step, in the order of [`STEPS`].
    pub millis: [f64; STEPS.len()],
}

/// Starts the clock of the chain.
fn start(mut laps: ResMut<Laps>) {
    laps.last = busy();
}

/// Notes what step `N` cost, the commands of the systems before it included.
fn lap<const N: usize>(mut laps: ResMut<Laps>) {
    let now = busy();
    laps.millis[N] = (now - laps.last) * 1000.0;
    laps.last = now;
}

/// Adds the streaming chain with a lap after every step, all on the calling thread.
fn chain(app: &mut App) {
    let planet = chunks::change_planet.run_if(resource_exists_and_changed::<Planet>);
    app.add_systems(
        Update,
        (
            (start, planet, chunks::follow_face, lap::<0>).chain(),
            (chunks::watch_far, lap::<1>).chain(),
            (chunks::grow, lap::<2>).chain(),
            (chunks::stream, lap::<3>).chain(),
            (chunks::collect, lap::<4>).chain(),
            (chunks::refine, lap::<5>).chain(),
            (chunks::show, lap::<6>).chain(),
            (chunks::mark_ready, chunks::place, lap::<7>).chain(),
        )
            .chain(),
    );
    app.edit_schedule(Update, |schedule| {
        schedule.set_executor(SingleThreadedExecutor::new());
    });
}

/// Returns an app with a new world begun, whose time advances one [`STEP`] per frame.
pub fn rig() -> App {
    let mut app = App::new();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut images = Assets::<Image>::default();
    app.add_plugins(MinimalPlugins)
        .insert_resource(PropAssets::new(&mut materials, &mut images))
        .insert_resource(materials)
        .insert_resource(images)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Chunks>()
        .init_resource::<Laps>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            STEP,
        )));
    let begun = app.world_mut().run_system_once(crate::world::begin_world);
    begun.expect("the world begins");
    chain(&mut app);
    app
}
