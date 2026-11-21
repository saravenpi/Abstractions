//! A glide over the base of a home: the wingsuit spread 300 m above it and flown hands off, in
//! the gravity and the air of the planet, as the controller hands them to the gear.

use bevy::prelude::*;

use crate::gear::{FlightState, GearInput, Mode, air_density, step};
use crate::universe::PlanetSpec;
use crate::wind::thickness;

/// Metres above the base the glide starts at.
const DROP: f32 = 300.0;

/// Metres of the drop flown before the glide is measured, and the metres it is measured over.
const WINDOW: (f32, f32) = (150.0, 100.0);

/// Frame time of the glide in seconds.
const DT: f32 = 1.0 / 60.0;

/// Most frames the glide may last: two minutes.
const FRAMES: u32 = 7200;

/// Fastest sink in metres per second of a flyable glide: the 11 m/s of a real suit, a landing
/// roll on touchdown, see `universe::GLIDE_AIR`.
pub const SINK: f32 = 11.0;

/// Fastest forward speed in metres per second of a flyable glide: the 50 m/s of a dive.
pub const FORWARD: f32 = 50.0;

/// Lowest glide ratio of a flyable glide, the low end of the tuned glide of four to one.
pub const RATIO: f32 = 3.5;

/// Returns the spread wings at the start of the glide: 38 m/s along a path 18 degrees down.
fn start() -> FlightState {
    FlightState {
        mode: Mode::Glide,
        velocity: Vec3::new(0.0, -12.0, -36.0),
        wings: 1.0,
        ..FlightState::default()
    }
}

/// What a hands-off glide over the base settles at.
#[derive(Debug, Clone, Copy, Default)]
pub struct Glide {
    /// Mean forward speed over the ground in metres per second.
    pub forward: f32,
    /// Mean sink rate in metres per second.
    pub sink: f32,
}

impl Glide {
    /// Returns the glide ratio: metres forward per metre down.
    pub fn ratio(&self) -> f32 {
        self.forward / self.sink.max(1e-3)
    }

    /// Returns true when the glide is inside the flyable envelope: [`SINK`], [`FORWARD`] and
    /// [`RATIO`].
    pub fn flyable(&self) -> bool {
        self.sink > 0.0 && self.sink <= SINK && self.forward <= FORWARD && self.ratio() >= RATIO
    }
}

/// Returns the hands-off glide of the wingsuit spread [`DROP`] metres above ground `base` metres
/// above the sea of `planet`, measured over the [`WINDOW`] of its drop.
pub fn glide(planet: &PlanetSpec, base: f32) -> Glide {
    let mut state = start();
    let input = GearInput {
        deploy: true,
        gravity: planet.gravity as f32,
        air: thickness(&planet.record) as f32,
        ..GearInput::default()
    };
    let top = planet.atmosphere_height as f32;
    let (mut drop, mut fallen, mut along, mut spent) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    for _ in 0..FRAMES {
        let density = air_density(top, base.max(0.0) + DROP - drop);
        state = step(&state, &GearInput { density, ..input }, DT);
        let fell = -state.velocity.y * DT;
        drop += fell;
        if drop > WINDOW.0 {
            (fallen, spent) = (fallen + fell, spent + DT);
            along += state.velocity.with_y(0.0).length() * DT;
        }
        if drop >= WINDOW.0 + WINDOW.1 {
            break;
        }
    }
    let time = spent.max(DT);
    Glide {
        forward: along / time,
        sink: fallen / time,
    }
}
