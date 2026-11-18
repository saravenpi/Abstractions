//! A gas giant seen from afar: latitude bands in the two tones of its class and one storm spot,
//! all drawn from its seed. `bodies.wesl` draws the bands from these values.

use std::f32::consts::TAU;

use bevy::math::{DVec3, Vec3};

use super::material::shares;
use crate::universe::{PlanetClass, PlanetRecord};

/// Fewest bands from pole to pole, and how many more the seed can add.
const BANDS: (f32, f32) = (7.0, 6.0);

/// How the storm tints the first tone: redder and darker, like a storm that dredges up deeper cloud.
const STORM_TINT: Vec3 = Vec3::new(1.05, 0.62, 0.42);

/// Smallest angular radius of the storm in radians, and how much larger the seed can make it.
const STORM_SIZE: (f32, f32) = (0.08, 0.1);

/// Farthest latitude in radians the storm sits at, north or south.
const STORM_LATITUDE: f32 = 0.45;

/// Salt that keeps the storm draws apart from the colour draws of the same seed.
const STORM_SALT: u64 = 0x2545_F491_4F6C_DD1D;

/// The bands and the storm of a gas giant. Colours are linear RGB.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Giant {
    /// Tone of the bright zones.
    pub first: Vec3,
    /// Tone of the dark belts.
    pub second: Vec3,
    /// Colour of the storm spot.
    pub storm: Vec3,
    /// Number of band waves from pole to pole.
    pub bands: f32,
    /// Phase in radians that shifts the bands toward a pole.
    pub phase: f32,
    /// Unit vector toward the centre of the storm, in planet-fixed axes.
    pub spot: DVec3,
    /// Cosine of the angular radius of the storm.
    pub size: f32,
}

impl Giant {
    /// Returns the bands and the storm of a planet, or nothing when it is not a gas giant.
    ///
    /// The tones are the land and the sea of the class look, which a gas giant fills with
    /// cloud colours, brightened or dimmed a little by the seed like every far planet.
    pub fn of(record: &PlanetRecord, seed: u64) -> Option<Self> {
        if record.class != PlanetClass::GasGiant {
            return None;
        }
        let look = record.class.look();
        let (count, shade) = shares(seed);
        let (along, high) = shares(seed ^ STORM_SALT);
        let bright = 0.9 + 0.2 * shade;
        let latitude = f64::from(STORM_LATITUDE * (2.0 * high - 1.0));
        let longitude = f64::from(TAU * along);
        let spot = DVec3::new(
            latitude.cos() * longitude.cos(),
            latitude.sin(),
            latitude.cos() * longitude.sin(),
        );
        Some(Self {
            first: look.land * bright,
            second: look.sea * bright,
            storm: look.land * bright * STORM_TINT,
            bands: BANDS.0 + (BANDS.1 * count).floor(),
            phase: TAU * shade,
            spot,
            size: (STORM_SIZE.0 + STORM_SIZE.1 * along).cos(),
        })
    }
}
