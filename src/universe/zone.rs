//! Starlight: how much of it reaches an orbit and where the orbit lies against the habitable zone.

use super::home::HOME_STAR_TEMPERATURE;
use super::record::Zone;

/// Orbit radius in metres where a star of luminosity 1 gives a flux of 1.
///
/// Orbits run from 5e6 to 5e8 m and three stars in four are red dwarfs of luminosity 0.05 to
/// 0.2, whose flux is 1 between 3.8e7 and 7.6e7 m: the habitable zone of the common star sits
/// in the middle of the orbit span, on a log scale, and every class of planet gets its share.
pub const FLUX_ORBIT: f64 = 1.7e8;

/// Coolest and hottest star temperature in kelvin the habitable zone fit holds for.
const FIT_SPAN: (f64, f64) = (2600.0, 7200.0);

/// Fit of the inner edge of the habitable zone, where a runaway greenhouse starts.
const INNER: [f64; 5] = [1.0512, 1.3242e-4, 1.5418e-8, -7.9895e-12, -1.8328e-15];

/// Fit of the outer edge of the habitable zone, where the strongest greenhouse still thaws water.
const OUTER: [f64; 5] = [0.3438, 5.8942e-5, 1.6558e-9, -3.0045e-12, -5.2983e-16];

/// Returns the starlight reaching an orbit of `orbit` metres, the home planet receiving 1.
///
/// Light thins as the square of the distance.
pub fn flux(luminosity: f64, orbit: f64) -> f64 {
    let reach = orbit / FLUX_ORBIT;
    luminosity / (reach * reach)
}

/// Returns the flux at one edge of the habitable zone of a star at `star_temperature` kelvin.
///
/// The edge is the polynomial `S + a T + b T^2 + c T^3 + d T^4` with `T` the star temperature
/// less 5,780 K. A cool star gives redder light that air and ice absorb better, so its edges
/// sit at a lower flux.
fn edge(fit: &[f64; 5], star_temperature: f64) -> f64 {
    let t = star_temperature.clamp(FIT_SPAN.0, FIT_SPAN.1) - HOME_STAR_TEMPERATURE;
    fit.iter().rev().fold(0.0, |sum, k| sum * t + k)
}

/// Returns where an orbit receiving `flux` lies against the habitable zone of its star.
///
/// The edges are the runaway greenhouse and maximum greenhouse limits of Kopparapu et al. 2013,
/// "Habitable Zones around Main-sequence Stars: New Estimates", The Astrophysical Journal 765,
/// 131, <https://iopscience.iop.org/article/10.1088/0004-637X/765/2/131>, with the coefficients
/// of <https://github.com/ageller/HZ_Kopparapu/blob/master/HZ_coefficients.dat>. The star
/// temperature is clamped to the 2,600 to 7,200 K the fit holds for.
pub fn zone(star_temperature: f64, flux: f64) -> Zone {
    if flux > edge(&INNER, star_temperature) {
        Zone::Hot
    } else if flux < edge(&OUTER, star_temperature) {
        Zone::Cold
    } else {
        Zone::Habitable
    }
}
