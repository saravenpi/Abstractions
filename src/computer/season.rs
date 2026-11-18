//! The season of a planet in plain words, and the span of its temperatures, for the card.
//!
//! The season is the share of the way round the star the planet has gone, as the climate field
//! reads it: at 0 the north leans toward the star, so it is the northern summer.

use std::f64::consts::TAU;

use bevy::math::{DVec3, Vec2};

use super::words::temperature;
use crate::climate::{self, Setting, Spot};
use crate::universe::PlanetSpec;

/// Seasons named in the order the north goes through them, from the start of the year.
const NORTH: [&str; 4] = ["summer", "autumn", "winter", "spring"];

/// Tilt in radians under which the year has no seasons worth a word: about 3 degrees.
const FLAT_TILT: f64 = 0.05;

/// Longitudes and latitudes the span of temperatures is sampled at.
const SAMPLES: (usize, usize) = (8, 9);

/// Returns the season of the north in plain words, such as "northern summer", or "no seasons"
/// when the spin axis stands almost straight.
pub fn season_words(tilt: f64, season: f32) -> String {
    if tilt.sin().abs() < FLAT_TILT.sin() {
        return "no seasons".to_string();
    }
    let quarter = ((season + 0.125).rem_euclid(1.0) * 4.0) as usize;
    format!("northern {}", NORTH[quarter.min(3)])
}

/// Returns the direction from the centre at `lat` radians north and `lon` radians east of +X,
/// +Y being the spin axis and east the way `+Y x dir` points.
pub fn direction(lon: f64, lat: f64) -> DVec3 {
    DVec3::new(lat.cos() * lon.cos(), lat.sin(), -lat.cos() * lon.sin())
}

/// Returns the coldest and the warmest sea level temperature of the planet in kelvin, over a
/// coarse grid that holds both poles and the equator.
pub fn span(setting: &Setting, season: f32) -> (f32, f32) {
    let (lons, lats) = SAMPLES;
    let mut span = (f32::INFINITY, f32::NEG_INFINITY);
    for i in 0..lons * lats {
        let lon = (i % lons) as f64 * TAU / lons as f64;
        let lat = ((i / lons) as f64 / (lats - 1) as f64 - 0.5) * std::f64::consts::PI;
        let spot = Spot {
            dir: direction(lon, lat),
            altitude: 0.0,
            inland: 0.0,
            slope: Vec2::ZERO,
        };
        let warmth = climate::at(setting, &spot, season).temperature;
        span = (span.0.min(warmth), span.1.max(warmth));
    }
    span
}

/// Returns a span of temperatures in kelvin as the card writes it: "-12 C to 31 C", or one
/// value when both ends round alike.
pub fn span_words((low, high): (f32, f32)) -> String {
    let (low, high) = (temperature(f64::from(low)), temperature(f64::from(high)));
    match low == high {
        true => format!("{low} everywhere"),
        false => format!("{low} to {high}"),
    }
}

/// Returns the climate line of the card for a planet `season` of the way round its year: the
/// season of the north and the span of temperatures, the near side in place of the season on
/// a moon that always shows its planet the same face.
pub fn climate_line(spec: &PlanetSpec, season: f32) -> String {
    let locked = spec.id.is_moon() && spec.day_seconds < 0.0;
    let lead = match locked {
        true => "near side warm".to_string(),
        false => season_words(spec.record.tilt, season),
    };
    let range = span_words(span(&Setting::of(spec), season));
    format!("{lead} - {range}")
}
