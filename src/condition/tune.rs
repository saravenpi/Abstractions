//! Tuning constants of the robot's condition. Health is a share from 0 (destroyed) to 1 (whole);
//! energies are joules.

use super::PARTS;

/// Energy that destroys each part from whole when it lands blunt on its plate, in joules, in the
/// order of [`super::Part::ALL`]: head, torso, arms, legs, wings, boots.
pub const PLATE: [f32; PARTS] = [
    900.0, 2400.0, 600.0, 600.0, 800.0, 800.0, 300.0, 300.0, 500.0, 500.0,
];

/// Share of a blunt blow's energy that gets through the plate and harms the part, unitless: the
/// shell spreads a blunt blow and dents.
pub const BLUNT: f32 = 0.6;

/// Share of a cutting blow's energy that harms the part, unitless: an edge or a point goes through
/// the plate.
pub const CUT: f32 = 1.0;

/// Health under which a part is shown as damaged and listed in the HUD.
pub const DAMAGED: f32 = 0.995;

/// Health under which a limb or a wing is broken and does no work, but dangles.
pub const BROKEN: f32 = 0.25;

/// Health under which a leg counts as badly hurt: with both under it the robot crawls.
pub const CRAWL: f32 = 0.35;

/// Least share of its walking speed a robot keeps with a hurt leg, unitless.
pub const LIMP_PACE: f32 = 0.45;

/// Share of its walking speed a robot keeps crawling, unitless.
pub const CRAWL_PACE: f32 = 0.2;

/// Least share of its jump a robot keeps with a hurt leg, unitless.
pub const LIMP_JUMP: f32 = 0.4;

/// Least share of its swing speed an arm keeps, unitless.
pub const ARM_POWER: f32 = 0.35;

/// Least share of its carrying capacity a robot keeps with a hurt torso, unitless.
pub const TORSO_CARRY: f32 = 0.5;

/// Least share of its swing recovery speed a robot keeps with a hurt torso, unitless.
pub const TORSO_RECOVERY: f32 = 0.5;

/// Least share of its companion scan reach a robot keeps with a hurt head, unitless.
pub const HEAD_SCAN: f32 = 0.4;

/// Least share of its lift a wing keeps, unitless.
pub const WING_LIFT: f32 = 0.2;

/// Least share of its thrust a boot keeps, unitless.
pub const BOOT_THRUST: f32 = 0.3;

/// Roll stick a fully lost wing pulls toward its own side, from -1 to 1.
pub const WING_PULL: f32 = 0.5;

/// Largest aim error of a throw with a fully broken arm, in radians.
pub const SHAKE: f32 = 0.25;
