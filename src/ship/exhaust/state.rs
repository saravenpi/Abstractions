//! What drives the exhaust of a ship, read from its real state: the main engines from the
//! level of its engine, the lift nozzles from the push of the vertical thrusters while the ship
//! hovers or flies slowly, the attitude jets from the stick and the turn the stabiliser still
//! has to make, the warp plume from the
//! spool of the drive, and the air around it from the planet and the height. Everything follows
//! the real value over the spool time of the engine, so a plume rises and dies, not blinks.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::dust::JETS;
use super::plume::ExhaustMaterial;
use super::shape::{ease, spool};
use super::tune::{CARRY, HEAT_DOWN, HEAT_UP, RCS_SPOOL};
use crate::ship::assist::TURN_RATE;
use crate::ship::body::Ship;
use crate::ship::cockpit::{Cockpit, Lamp};
use crate::ship::flight::Report;
use crate::ship::glow::Engine;
use crate::ship::helm::Helm;
use crate::ship::sim::Pace;
use crate::ship::sky::Skies;
use crate::ship::tune::LIFT_ACCEL;
use crate::ship::wells::beside;
use crate::ship::{SEA_DENSITY, density};
use crate::space::{Anchor, Frame, Placed};

/// Turn rate error in radians per second at which an attitude jet fires fully.
const ERROR: f64 = 0.3;

/// The exhaust of one ship: how far each kind of jet is up, spooled from the real state, and the
/// materials the ship draws its plumes with.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Exhaust {
    /// Thrust of the main engines, 1 at full throttle, more with the boost.
    pub main: f32,
    /// Thrust of the lift nozzles, 1 at their full push.
    pub lift: f32,
    /// How far the warp drive is up, from 0 to 1.
    pub warp: f32,
    /// Glow of the nozzles, from 0 to 1: it follows the thrust up and cools slowly after.
    pub heat: f32,
    /// Fire of the attitude jets on each axis, from -1 to 1: pitch up, yaw left, roll left.
    pub rcs: Vec3,
    /// The air around the ship as a share of the air at sea level at home.
    pub air: f32,
    /// Puffs of dust each jet owes the ground: the four lift nozzles, then the two main engines.
    pub dust: [f32; JETS],
    /// The materials of the plumes: the main engines, the lift nozzles and the attitude jets.
    pub jets: [Handle<ExhaustMaterial>; 3],
}

/// What the state is read from.
#[derive(SystemParam)]
pub struct Scene<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// The frame of the render anchor.
    anchor: Res<'w, Anchor>,
    /// The planet under the ship.
    skies: Res<'w, Skies>,
    /// What the last flight step did.
    pace: Res<'w, Pace>,
    /// What the pilot asks of the ship.
    helm: Res<'w, Helm>,
}

/// Returns the share of the push across the nose of a ship that its lift nozzles give, when
/// it flies `forward` metres per second along its nose: all of it in a hover, none once the
/// wing carries the ship in air, or its main engines and the attitude of its hull in space.
/// The share eases across [`CARRY`], so the nozzles fade out and never pop.
pub fn hovering(forward: f64) -> f64 {
    let x = ((forward - CARRY.0) / (CARRY.1 - CARRY.0)).clamp(0.0, 1.0);
    1.0 - x * x * (3.0 - 2.0 * x)
}

/// Returns the thrust of the lift nozzles of a ship, as a share of their full push: the push
/// along its roof from the step report, times the share of it they give at the speed along the
/// nose of `vel`, its velocity against the ground or the body it is beside (see
/// [`hovering`]). A pitch in forward flight turns the path with the wing or the hull, so it
/// fires the attitude jets and never the lift nozzles. On the gear it is what the pilot asks
/// of the lever, `asked`, because the push the crew feels there is the ground's. They are shut
/// when the ship rests.
pub fn lift(ship: &Ship, report: &Report, (asked, vel): (f64, DVec3)) -> f32 {
    if ship.landed {
        return 0.0;
    }
    let hover = hovering(vel.dot(ship.forward()));
    if ship.touching {
        return (asked.clamp(0.0, 1.0) * hover) as f32;
    }
    let push = (ship.rot.inverse() * report.proper).y / LIFT_ACCEL;
    (push.clamp(0.0, 1.5) * hover) as f32
}

/// Returns the thrust of the lift nozzles of a ship of another player, whose record carries
/// no push: the share of its weight under `gravity` they hold at the speed its instruments
/// read (see [`hovering`]), and nothing when it is landed, in warp or away from a planet.
pub fn held(cockpit: &Cockpit, gravity: f64) -> f32 {
    let landed = cockpit.read.lamps[Lamp::Landed as usize];
    let Some(speed) = cockpit.read.speed.filter(|_| !landed) else {
        return 0.0;
    };
    (hovering(speed) * gravity / LIFT_ACCEL) as f32
}

/// Returns the velocity of a ship against the ground under it, or in space against the body
/// it is beside.
fn moving(skies: &Skies, ship: &Ship) -> DVec3 {
    let Frame::System(_) = ship.frame else {
        return ship.vel;
    };
    beside(&skies.wells, ship.pos).map_or(ship.vel, |well| ship.vel - well.vel)
}

/// Returns the fire of the attitude jets on each axis. With the assist the jets close the gap
/// between the turn the stick asks for and the turn the ship has; without it they follow the stick.
pub fn demand(stick: Vec3, ship: Option<&Ship>) -> Vec3 {
    match ship.filter(|ship| ship.assist) {
        Some(ship) => ((stick.as_dvec3() * TURN_RATE - ship.spin) / ERROR)
            .clamp(DVec3::NEG_ONE, DVec3::ONE)
            .as_vec3(),
        None => stick,
    }
}

/// Returns the air at a ship placed at `placed` as a share of the air at sea level at home,
/// and the surface gravity of the planet in metres per second squared: neither away from a
/// planet.
fn air(scene: &Scene, placed: &Placed) -> (f32, f64) {
    let (Frame::Surface(_), Some(planet)) = (scene.anchor.frame, scene.skies.planet()) else {
        return (0.0, 0.0);
    };
    let thick = density(planet, placed.pos.length() - planet.radius) / SEA_DENSITY;
    (thick as f32, planet.gravity)
}

/// The parts of a ship the exhaust is read from.
type Hulls = (
    &'static Engine,
    &'static Cockpit,
    &'static Placed,
    Option<&'static Ship>,
    &'static mut Exhaust,
);

/// Spools the exhaust of every ship toward the real state of its engines, for the frame.
pub fn feed(scene: Scene, mut ships: Query<Hulls>) {
    let dt = scene.time.delta_secs().min(0.1);
    for (engine, cockpit, placed, ship, mut exhaust) in &mut ships {
        let thrust = if engine.warp > 0.0 { 0.0 } else { engine.level };
        let (thick, gravity) = air(&scene, placed);
        let asked = scene.helm.controls.lift;
        let pushed = ship.map_or_else(
            || held(cockpit, gravity),
            |ship| {
                lift(
                    ship,
                    &scene.pace.report,
                    (asked, moving(&scene.skies, ship)),
                )
            },
        );
        let jets = demand(cockpit.asked.stick, ship);
        let share = 1.0 - (-dt / RCS_SPOOL).exp();
        exhaust.main = spool(exhaust.main, thrust, dt);
        exhaust.lift = spool(exhaust.lift, pushed, dt);
        exhaust.warp = ease(exhaust.warp, engine.warp, (0.25, 0.8), dt);
        let burning = exhaust.main.min(1.0).max(exhaust.lift.min(1.0) * 0.8);
        exhaust.heat = ease(
            exhaust.heat,
            burning.max(exhaust.warp),
            (HEAT_UP, HEAT_DOWN),
            dt,
        );
        exhaust.rcs = exhaust.rcs.lerp(jets, share);
        exhaust.air = thick;
    }
}
