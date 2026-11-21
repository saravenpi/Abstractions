//! The Earth-like home: the air, the pull and the warmth of the planet a new world starts on.
//!
//! A home of 0.11.0 only had to be [`homely`]: half of them had under 0.23 bar of air, too thin
//! for the wingsuit to glide and far from the Earth. A new world now starts on the nearest
//! homely planet whose air, pull and warmth are close to the Earth's, see [`earthlike`].

use super::air::Gas;
use super::home::{Fit, SEARCH, homely};
use super::spec::PlanetSpec;
use super::units::STANDARD_GRAVITY;
use super::{Universe, home};
use crate::life::{Rung, World, chem};

/// The home rule new worlds are made with: 0 is the first rule, of 0.11.0, where a home is
/// [`homely`], and 1 the rule of the Earth-like home, where it is [`earthlike`]. A save keeps the rule of the world it
/// holds, so a world made before the rule changed keeps its home.
pub const HOME_RULE: u32 = 1;

/// Systems the search for an Earth-like home looks at before it settles for a homely planet:
/// eight times [`SEARCH`], since about one homely planet in five has air like the Earth's and
/// about one in three of those a pull within [`PULL`].
pub const REACH: usize = 8 * SEARCH;

/// Lowest and highest air pressure at sea level of an Earth-like home, the Earth having 1.
pub const PRESSURE: (f64, f64) = (0.7, 1.5);

/// Lowest and highest surface gravity in metres per second squared of an Earth-like home: 0.85
/// and 1.05 standard gravities, a little on the light side of the Earth, so the robot takes off
/// and the ship lifts easily.
pub const PULL: (f64, f64) = (0.85 * STANDARD_GRAVITY, 1.05 * STANDARD_GRAVITY);

/// Coldest and warmest mean surface temperature in kelvin of an Earth-like home: 5 to 30 C,
/// around the 15 C of the Earth.
pub const TEMPERATE: (f64, f64) = (278.15, 303.15);

/// Least share of oxygen in the air the life of an Earth-like home breathes: life that splits
/// water fills it to the 0.21 of the Earth, see `life::chem::air`.
pub const BREATH: f64 = 0.15;

/// Returns true when `planet`, whose life climbed to `rung` in `world`, is a home like the
/// Earth: [`homely`], with air of [`PRESSURE`] at sea level, a pull within [`PULL`], a mean of
/// [`TEMPERATE`], and air of nitrogen with oxygen its life made, [`BREATH`] of it at least.
///
/// The wingsuit then glides with room to spare. It needs a bite of the air of
/// `(TRIM_SINK / MOST_SINK)^2 g / g0`, 0.434 of the home air per standard gravity, see
/// `gear::glides`. The thinnest air these bounds let through, 0.7 bar of nitrogen with a
/// hundredth of carbon dioxide and of argon at 30 C, is 0.65 of the home air, see
/// [`super::dense`], and the strongest pull, 1.05 g, asks 0.46: two fifths more air than the
/// glide needs, more under a lighter pull. Water boils at 90 C under 0.7 bar, so the seas stay liquid.
pub(super) fn earthlike(planet: &PlanetSpec, rung: Rung, world: &World) -> bool {
    let record = &planet.record;
    let within = |(low, high): (f64, f64), value: f64| (low..=high).contains(&value);
    let breathed = chem::air(&chem::chemistry(world, planet.seed), record.mix);
    let air = breathed.main_gas() == Gas::Nitrogen && breathed.share(Gas::Oxygen) >= BREATH;
    let like = within(PRESSURE, record.pressure)
        && within(PULL, planet.gravity)
        && within(TEMPERATE, record.temperature);
    like && air && homely(planet, rung, world)
}

/// Returns the test a home passes under the home `rule` and the systems the search looks at for
/// one: [`homely`] and [`SEARCH`] under rule 0, [`earthlike`] and [`REACH`] under any later one.
pub(super) fn rule(rule: u32) -> (Fit, usize) {
    match rule {
        0 => (homely as Fit, SEARCH),
        _ => (earthlike as Fit, REACH),
    }
}

impl Universe {
    /// Builds the universe for a seed under the home `rule` of a world: 0 for a world made
    /// before the home had to be Earth-like, [`HOME_RULE`] for a new one. Only the home differs
    /// between rules: every galaxy, star and planet is the same.
    pub fn under(seed: u64, rule: u32) -> Self {
        let home = home::choose(seed, rule);
        Self { seed, home }
    }
}
