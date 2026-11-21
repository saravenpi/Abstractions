//! Bumps: a blow stops on what it meets in the world. After the transforms are propagated the
//! blade of every swing and stab in flight is swept from its last frame to this one against the
//! ground, the drawn ground where no collider is loaded, rocks, trees, the base and the ship. The
//! first contact stops the tool there: the hands stagger back off it by the bounce of the contact,
//! the blow sounds and throws dust or sparks by what it struck, and its energy goes into it. An
//! animal in the way has already taken the blow, as [`super::sense`] lays out.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::act::{Act, Strike, Timing};
use super::held::Held;
use super::javelin::Soil;
use super::javelin::tune::ROCK;
use super::outcome::{Outcome, rebound};
use super::path::Path;
use super::sense::landed;
use super::solid::{Land, Meets, Solids, Touch};
use super::tune::{BOUNCE, GLANCE, GROUND, REBOUND, SAMPLES, STAGGER};
use crate::audio::{Body, Impact, Outer};

/// Returns where the blade that moved from `was` to `now`, grip and tip in render space, first
/// meets `solids`, and how far through the move it does, 0 at `was` to 1 at `now`: the path of
/// each point of [`SAMPLES`] along the blade is swept, and the earliest contact wins.
pub fn swept(solids: &impl Meets, (was, now): ([Vec3; 2], [Vec3; 2])) -> Option<Touch> {
    let point = |blade: [Vec3; 2], s: f32| blade[0].lerp(blade[1], s);
    SAMPLES
        .iter()
        .filter_map(|s| solids.meet(point(was, *s), point(now, *s)))
        .min_by(|a, b| a.share.total_cmp(&b.share))
}

/// Returns the act of a swing timed by `timing` that met the world `share` of the way through its
/// last `dt` seconds, `t` seconds in, striking stuff as stiff as `struck` with a head as stiff as
/// `tip`: it staggers back from the share of the path where it met, by the bounce of the contact.
pub fn stopped((t, dt): (f32, f32), timing: &Timing, share: f32, (struck, tip): (f32, f32)) -> Act {
    let length = timing.swing.max(f32::EPSILON);
    let met = ((t - dt * (1.0 - share)) / length).clamp(0.0, 1.0);
    let back = REBOUND * rebound(struck, tip) / BOUNCE;
    Act::Stun(STAGGER, (met - back).max(0.0))
}

/// Returns the impact of `held` meeting the world at `touch` along the unit `along` with `energy`
/// joules: soil thuds and throws dust, rock clacks and sparks under a hard head.
fn bumped(held: &Held, touch: &Touch, (along, energy): (Vec3, f32)) -> Impact {
    let (stiffness, mass) = match touch.soil {
        Soil::Soft(_) => GROUND,
        Soil::Hard => ROCK,
    };
    Impact {
        at: touch.at,
        joules: energy,
        cut: false,
        outer: Outer::Ground,
        struck: Body { stiffness, mass },
        tool: Body {
            stiffness: held.build.head.material.stiffness,
            mass: held.build.head.mass,
        },
        deflected: along.dot(touch.normal).abs() < GLANCE,
    }
}

/// Stops every swing and stab in flight that meets the world this frame where it meets it, and
/// sounds the blow there.
pub fn block(
    (time, land): (Res<Time>, Land),
    world: SpatialQuery,
    mut bodies: Query<(&GlobalTransform, &mut Strike, &Held, &ChildOf)>,
    mut sounds: MessageWriter<Impact>,
) {
    let dt = time.delta_secs();
    for (placed, mut strike, held, of) in &mut bodies {
        let Act::Swing(t) = strike.act else { continue };
        if strike.landed {
            continue;
        }
        let in_world = |pair: [Vec3; 2]| pair.map(|p| placed.transform_point(p));
        let (now, was) = (in_world(strike.blade[0]), in_world(strike.blade[1]));
        let solids = Solids::new(&world, &land, of.parent());
        let Some(touch) = swept(&solids, (was, now)) else {
            continue;
        };
        let timing = Timing::of(held, &Path::of(held.style, strike.dir)).felt(&strike.hurt);
        let struck = [GROUND.0, ROCK.0][usize::from(touch.soil == Soil::Hard)];
        let tip = held.build.head.material.stiffness;
        let along = (now[1] - was[1]).normalize_or_zero();
        let impact = bumped(held, &touch, (along, landed(held, now, touch.at).0.energy));
        strike.act = stopped((t, dt), &timing, touch.share, (struck, tip));
        strike.landed = true;
        strike.result = match impact.deflected {
            true => Outcome::Deflected,
            false => Outcome::Stopped,
        };
        sounds.write(impact);
    }
}
