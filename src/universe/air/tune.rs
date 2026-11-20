//! Numbers of the air model: physical constants first, then the shares and limits that are tuned.

/// Molar gas constant in joules per mole and kelvin.
pub const GAS_CONSTANT: f64 = 8.314;

/// Escape speed of the home planet in metres per second: the Earth's, which the home stands for.
pub const HOME_ESCAPE: f64 = 11_186.0;

/// Mean surface temperature of the home planet in kelvin.
pub const HOME_TEMPERATURE: f64 = 288.0;

/// Scale heights between the sea level and the top of the air.
///
/// At 12 scale heights 6 millionths of the air are left: about 100 km on the Earth, where space
/// is said to begin.
pub const TOP_SCALES: f64 = 12.0;

/// Smallest planet radius over scale height at which a planet still holds its air.
///
/// The radius over the scale height is the escape parameter `G M m / (k T R)`. Fossati et al.
/// 2017, <https://arxiv.org/abs/1612.05624>, find that air boils off a planet when it falls
/// under 15 to 35, counted for hydrogen atoms; the same limit is taken here for the whole mix,
/// so no air is ever puffed up further. The Earth stands at about 750.
pub const LOOSEST: f64 = 20.0;

/// Thermal speeds a planet's escape speed must exceed to keep a gas over its life, as a ratio.
///
/// The usual rule of thumb: at 6 times the mean thermal speed the fast tail that leaves is
/// negligible over billions of years.
pub const JEANS: f64 = 6.0;

/// Pressure in home atmospheres at which the weather layer ends and clouds top out.
///
/// Thick atmospheres all turn stable near 0.1 bar (Robinson and Catling 2014,
/// <https://arxiv.org/abs/1312.6859>). The model cools rising air at the dry rate, faster than
/// real moist air, so it reaches that layer too low: 0.2 puts the Earth's at its real 11 km.
pub const TROPOPAUSE: f64 = 0.2;

/// Relative humidity of the air at the ground, from 0 to 1, on a world with no liquid and on a
/// world all under liquid.
pub const HUMIDITY: (f64, f64) = (0.3, 0.8);

/// Heat of vaporisation of a liquid over the gas constant times its boiling point, as a ratio.
///
/// Trouton's rule: about 10.5 for most liquids. It stands in for the liquid of a world whose
/// clouds are not water.
pub const TROUTON: f64 = 10.5;

/// Largest share of the cloud top height at which the cloud base sits.
pub const BASE_SHARE: f64 = 0.5;

/// Kelvin at which carbon dioxide frost forms under one home atmosphere.
pub const CARBON_FROST: f64 = 194.7;

/// Gas constant over the heat of sublimation of carbon dioxide, per kelvin: 8.314 over
/// 26,100 J/mol.
pub const CARBON_VAPOUR: f64 = 3.19e-4;

/// Inner heat, in units of the starlight the home planet gets, from which a moon is volcanic:
/// the 2.5 W/m2 of Io over the 1,361 W/m2 of sunlight at the Earth.
pub const VOLCANIC: f64 = 0.002;

/// Share of oxygen in air kept by life that splits water, as on the Earth.
pub const OXYGEN: f64 = 0.2095;

/// Smallest and largest share of helium in a gas giant: Saturn has 0.03, Neptune 0.19.
pub const HELIUM: (f64, f64) = (0.03, 0.19);

/// Share of methane in a gas giant: Jupiter has 0.003, Saturn 0.0045.
pub const GIANT_METHANE: f64 = 0.004;

/// Smallest and largest share of sulphur dioxide over molten or volcanic ground: Io has 0.9.
pub const SULPHUR: (f64, f64) = (0.5, 0.9);

/// Share of nitrogen over molten or volcanic ground.
pub const SULPHUR_NITROGEN: f64 = 0.03;

/// Smallest and largest share of methane in cold nitrogen air: Titan has 0.014 high up and
/// 0.05 at the ground.
pub const METHANE: (f64, f64) = (0.01, 0.06);

/// Share of argon in the air of a solid planet: the Earth has 0.009, Mars 0.016.
pub const ARGON: f64 = 0.01;

/// Smallest and largest share of carbon dioxide left in the air of a world with liquid water
/// and no life, where the sea locks most of it into rock.
pub const WET_CARBON: (f64, f64) = (0.01, 0.15);

/// Smallest and largest share of steam in the air of a world whose water has boiled.
pub const STEAM: (f64, f64) = (0.1, 0.4);

/// Smallest and largest share of nitrogen in carbon dioxide air: Mars has 0.027, Venus 0.035.
pub const DRY_NITROGEN: (f64, f64) = (0.02, 0.05);
