//! A source that churns the sea: it blows on it, skims it or jets under it, and the water answers
//! with spray, bubbles, a wake and ripple rings.
//!
//! Everything is thrown per metre travelled, with a rate per second added for a source that
//! hovers, so the same flight leaves the same water at any frame rate.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::rng::Rng;
use super::drops::{Drop, Drops};
use super::foam::{Foam, FoamMesh};
use super::throw::{Hand, Mark, Spot, ring, wake};
use super::tune::{
    BUBBLES_PER_METRE, BUBBLES_PER_SECOND, JUMP, RIPPLE_PERIOD, SPRAY_PER_METRE, SPRAY_PER_SECOND,
};
use super::wash::{Flow, flow, ripple_speed, skim, thrown};
use super::{trail, wet};
use crate::render::smoothstep;
use crate::space::{Anchor, Clock, altitude, up};
use crate::world::wake::Wakes;
use crate::world::{Globe, Liquid, Terrain};

/// What a source remembers between two frames.
#[derive(Debug, Default, Clone, Copy)]
struct Trace {
    /// Frame position on the frame before.
    was: Option<DVec3>,
    /// Spray flecks owed, as a fraction.
    spray: f32,
    /// Bubbles owed, as a fraction.
    bubbles: f32,
    /// Seconds since the last ripple ring.
    rings: f32,
    /// The last point of its wake.
    mark: Option<Mark>,
    /// Seconds since the last packet of waves.
    waked: f32,
}

/// A body that churns the sea under it. Its owner sets the first four fields every frame.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Churn {
    /// Force in newtons it pushes on the air, normally its weight in steady flight.
    pub push: f32,
    /// Size in metres of the disc it pushes the air with.
    pub size: f32,
    /// Density in kilograms per cubic metre of the air around it.
    pub air: f32,
    /// How hard a water jet runs under the sea, from 0 to 1.
    pub jet: f32,
    /// What it remembers between frames.
    trace: Trace,
}

impl Trace {
    /// Returns the velocity in render axes and the metres travelled since the frame before, from
    /// the render position `at` after `dt` seconds. A jump starts the trace afresh.
    fn travel(&mut self, anchor: &Anchor, at: Vec3, dt: f32) -> (Vec3, f32) {
        let here = anchor.to_frame(at);
        let step = here - self.was.replace(here).unwrap_or(here);
        let metres = step.length() as f32;
        if metres > JUMP {
            self.mark = None;
            return (Vec3::ZERO, 0.0);
        }
        ((anchor.rotation * step).as_vec3() / dt, metres)
    }
}

/// What the churn systems read and write.
#[derive(SystemParam)]
pub struct Sea<'w, 's> {
    /// Frame clock.
    time: Res<'w, Time>,
    /// Universe time, which the waves and the foam count in.
    clock: Res<'w, Clock>,
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// The terrain, for the planet.
    terrain: Res<'w, Terrain>,
    /// The foam.
    foam: ResMut<'w, Foam>,
    /// The waves the sources leave in the sea.
    wakes: ResMut<'w, Wakes>,
    /// The meshes, for the foam.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// The pool order of the drops.
    drops: ResMut<'w, Drops>,
    /// Randomness.
    rng: ResMut<'w, Rng>,
    /// The pooled drops.
    pool: Query<'w, 's, (&'static mut Drop, &'static mut Transform)>,
}

/// What lays the water of one frame.
struct Work<'a, 'w, 's> {
    /// Throws the drops.
    hand: Hand<'a, 'w, 's>,
    /// The meshes, for the foam.
    meshes: &'a mut Assets<Mesh>,
    /// The foam.
    foam: &'a mut Foam,
    /// The waves the sources leave in the sea.
    wakes: &'a mut Wakes,
    /// Density in kilograms per cubic metre of the liquid of the sea.
    liquid: f32,
    /// Where the foam mesh stands in render space.
    origin: Vec3,
    /// The render anchor.
    anchor: &'a Anchor,
    /// Seconds of the frame.
    dt: f32,
}

impl Work<'_, '_, '_> {
    /// Throws the spray of `spot` and returns how strongly it sprays, from 0 to 1.
    fn spray(&mut self, trace: &mut Trace, (spot, flow): (&Spot, &Flow), metres: f32) -> f32 {
        let strength = flow
            .spray
            .max(skim(spot.velocity.length(), spot.height, spot.size));
        let amount = SPRAY_PER_METRE * metres + SPRAY_PER_SECOND * flow.spray * self.dt;
        self.hand
            .spray(spot, flow, thrown(&mut trace.spray, strength * amount));
        strength
    }

    /// Lets the bubbles of a jet of strength `jet` go when `spot` is under the sea.
    fn bubbles(&mut self, trace: &mut Trace, (spot, jet): (&Spot, f32), metres: f32) {
        if spot.height >= 0.0 || jet <= 0.0 {
            trace.bubbles = 0.0;
            return;
        }
        let slow = 1.0 - smoothstep(0.0, 2.0, spot.velocity.length());
        let amount = BUBBLES_PER_METRE * metres + BUBBLES_PER_SECOND * slow * self.dt;
        self.hand
            .bubbles(spot, thrown(&mut trace.bubbles, jet * amount));
    }

    /// Lays the wake of `spot` while it sprays and moves, and breaks it off otherwise.
    fn wake(&mut self, trace: &mut Trace, spot: &Spot, strength: f32) {
        trace.mark = match strength > 0.0 && spot.velocity.length() > 1.0 {
            true => {
                let lay = (&mut *self.meshes, &mut *self.foam, self.origin);
                wake(lay, self.anchor, (spot, strength), trace.mark)
            }
            false => None,
        };
    }

    /// Starts a ripple ring under `spot` every [`RIPPLE_PERIOD`] while its wash hovers over the sea.
    fn rings(&mut self, trace: &mut Trace, (spot, flow): (&Spot, &Flow)) {
        if flow.ripple <= 0.0 || spot.velocity.length() > 2.0 {
            trace.rings = 0.0;
            return;
        }
        trace.rings += self.dt;
        if trace.rings >= RIPPLE_PERIOD {
            trace.rings %= RIPPLE_PERIOD;
            let lay = (&mut *self.meshes, &mut *self.foam, self.origin);
            ring(lay, spot, (ripple_speed(spot.gravity), flow.ripple));
        }
    }
}

/// Returns the spot of a source at render position `at` moving at `velocity`, without its size
/// and clock.
fn spot(anchor: &Anchor, globe: &Globe, (at, velocity): (Vec3, Vec3)) -> Spot {
    let up = up(anchor, at);
    Spot {
        at,
        up,
        height: altitude(anchor, globe, at),
        gravity: globe.gravity() as f32,
        velocity: velocity - up * velocity.dot(up),
        size: 0.0,
        now: 0.0,
    }
}

impl Work<'_, '_, '_> {
    /// Churns the sea under one source that travelled `metres` to `spot` this frame.
    fn tend(&mut self, churn: &mut Churn, spot: &Spot, metres: f32) {
        let speed = spot.velocity.length();
        let wash = flow(churn.push, churn.size, churn.air, (speed, spot.height));
        let strength = self.spray(&mut churn.trace, (spot, &wash), metres);
        self.bubbles(&mut churn.trace, (spot, churn.jet), metres);
        self.wake(&mut churn.trace, spot, strength);
        self.rings(&mut churn.trace, (spot, &wash));
        let density = ((churn.air, self.liquid), self.dt);
        let cause = (spot, &wash, strength);
        trail::lay(
            self.wakes,
            self.anchor,
            &mut churn.trace.waked,
            cause,
            density,
        );
    }
}

/// Churns the sea under every source: spray, bubbles, wakes and ripple rings. A source over dry
/// land churns nothing: its flow meets the ground, which raises dust or nothing, never spray.
pub fn churn(
    mut sea: Sea,
    mut sources: Query<(&mut Churn, &Transform), Without<Drop>>,
    foam_at: Query<&Transform, (With<FoamMesh>, Without<Drop>)>,
) {
    let dt = sea.time.delta_secs();
    let origin = foam_at.iter().next().map(|at| at.translation);
    let Some(origin) = origin.filter(|_| dt > 0.0) else {
        return;
    };
    let now = sea.clock.phase();
    let mut work = Work {
        hand: Hand {
            drops: &mut sea.drops,
            pool: &mut sea.pool,
            rng: &mut sea.rng,
        },
        meshes: &mut sea.meshes,
        foam: &mut sea.foam,
        wakes: &mut sea.wakes,
        liquid: Liquid::of(sea.terrain.globe()).density,
        origin,
        anchor: &sea.anchor,
        dt,
    };
    for (mut churn, at) in &mut sources {
        let churn = &mut *churn;
        let (velocity, metres) = churn.trace.travel(work.anchor, at.translation, dt);
        let found = spot(work.anchor, sea.terrain.globe(), (at.translation, velocity));
        let size = churn.size;
        match wet(&sea.terrain, work.anchor, at.translation) {
            true => work.tend(churn, &Spot { size, now, ..found }, metres),
            false => churn.trace.mark = None,
        }
    }
}
