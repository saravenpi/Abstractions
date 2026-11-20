//! Swing: how fast a build moves in the hand, and what it carries into a blow.

use super::inertia::{inertia, mass, moment};
use super::tune::{ARM_MASS, HAND_INERTIA, SWING_ARC, THRUST_ENERGY, TIP_C, TIP_POWER};
use super::{Build, Place};

/// What a build does in the hand on one world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swing {
    /// Moment of inertia about the hand, in kg·m².
    pub inertia: f32,
    /// Speed of the head's centre at the end of a swing, in metres per second.
    pub speed: f32,
    /// Angular speed at the end of a swing, in radians per second.
    pub spin: f32,
    /// Kinetic energy ½Iω² at the end of a swing, in joules.
    pub energy: f32,
    /// Momentum at the head, I·ω / r, in kilogram metres per second.
    pub momentum: f32,
    /// Torque to hold it level, g·Σm·r, in newton metres.
    pub torque: f32,
    /// Seconds from rest to the blow, stretched by torque over the grip budget.
    pub windup: f32,
    /// Speed of a thrust, in metres per second.
    pub thrust: f32,
}

/// Returns the speed of the head at the end of a swing for a build of `inertia` kg·m² about the hand,
/// v = C / (I + I_hand)^0.27, in metres per second.
///
/// The law is measured on swung rackets and bats, with speed hardly changing with mass at a fixed I
/// (Cross and Bower,
/// https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=DOI:%2210.1080/02640410500127876%22&resultType=core&format=json;
/// Schorah et al.,
/// https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=DOI:%2210.1080/14763141.2015.1027949%22&resultType=core&format=json).
pub fn tip_speed(inertia: f32) -> f32 {
    TIP_C / (inertia + HAND_INERTIA).powf(TIP_POWER)
}

/// Returns the swing of `build` at `place`.
///
/// The head reaches the tip speed at its centre. The swing accelerates evenly over a quarter turn, so
/// it takes 2·arc / ω; above the grip budget it stretches by torque / budget. A thrust puts a fixed
/// arm work into the build and the arm behind it, so its speed falls as 1 / √(m + m_arm).
pub fn swing(build: &Build, place: &Place) -> Swing {
    let pieces = build.pieces();
    let inertia = inertia(&pieces);
    let speed = tip_speed(inertia);
    let radius = build.shaft.length.max(f32::EPSILON);
    let spin = speed / radius;
    let torque = place.gravity * moment(&pieces);
    let stretch = (torque / build.hands.budget()).max(1.0);
    Swing {
        inertia,
        speed,
        spin,
        energy: 0.5 * inertia * spin * spin,
        momentum: inertia * spin / radius,
        torque,
        windup: 2.0 * SWING_ARC / spin * stretch,
        thrust: (2.0 * THRUST_ENERGY / (mass(&pieces) + ARM_MASS)).sqrt(),
    }
}
