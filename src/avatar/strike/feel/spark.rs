//! Flecks at the contact of a blow: every [`Impact`] heard throws them from where it landed, by
//! what it struck. Hide sheds tufts, shell and plate throw flakes and sparks, bone and horn pale
//! chips and dust, soil and stone dust and grit, and stone sparks under a hard head. Flesh throws
//! nothing here: its blood does. The impacts of the other players come through the same message,
//! so everyone sees the same flecks.

use bevy::prelude::*;

use super::tune::{FLECKS, RADII, SPARK, SPARKING, TONES};
use crate::audio::{Impact, Outer};
use crate::condition::luck;
use crate::effects::Fling;
use crate::space::{Anchor, up};

/// One kind of fleck a contact throws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spray {
    /// How many.
    pub count: u32,
    /// Speed they leave at, in metres per second.
    pub speed: f32,
    /// Radius of each, in metres.
    pub radius: f32,
    /// Linear RGB colour, above 1 for a spark.
    pub colour: [f32; 3],
}

/// Returns the stiffness of the contact of `impact`, in gigapascals: two springs in series.
fn contact(impact: &Impact) -> f32 {
    let (a, b) = (impact.struck.stiffness, impact.tool.stiffness);
    1.0 / (1.0 / a.max(1e-3) + 1.0 / b.max(1e-3))
}

/// Returns the flecks `impact` throws, at most two kinds: more and faster for more energy, sparks
/// only off a hard contact, nothing off flesh.
pub fn sprays(impact: &Impact) -> [Option<Spray>; 2] {
    let root = impact.joules.max(0.0).sqrt();
    let count = (root / 1.5).clamp(1.0, FLECKS) as u32;
    let speed = (1.5 + 0.25 * root).min(9.0);
    let spray = |n: u32, fast: f32, (radius, colour): (f32, [f32; 3])| Spray {
        count: n.max(1),
        speed: speed * fast,
        radius,
        colour,
    };
    let sparks = contact(impact) >= SPARKING;
    let spark = sparks.then(|| spray(count, 2.0, (RADII[3], SPARK)));
    match impact.outer {
        Outer::Flesh => [None, None],
        Outer::Hide => [Some(spray(count, 0.5, (RADII[1], TONES[1]))), None],
        Outer::Shell | Outer::Plate => [Some(spray(count / 2, 1.0, (RADII[2], TONES[3]))), spark],
        Outer::RobotPlate => [Some(spray(count / 2, 1.0, (RADII[2], TONES[4]))), spark],
        Outer::Bone | Outer::Horn => [
            Some(spray(count / 2, 1.0, (RADII[2], TONES[2]))),
            Some(spray(count / 2, 0.4, (RADII[0], TONES[2]))),
        ],
        Outer::Ground => [Some(spray(count, 0.5, (RADII[0], TONES[0]))), spark],
    }
}

/// Returns the velocity of fleck `n` of `spray` thrown from a surface whose up is `up`, scattered
/// over a wide cone by the roll `key`.
pub fn velocity(spray: &Spray, up: Vec3, (n, key): (u32, u32)) -> Vec3 {
    let roll = |salt: u32| luck(key ^ n.wrapping_mul(0x9e37) ^ salt) * 2.0 - 1.0;
    let side = Vec3::new(roll(0x51), roll(0xa3), roll(0x7f)) * 1.4;
    let dir = (up + side).normalize_or(up);
    dir * spray.speed * (0.6 + 0.4 * luck(key ^ n ^ 0x3c))
}

/// Throws the flecks of every impact heard from its contact point.
pub fn scatter(
    mut impacts: MessageReader<Impact>,
    (anchor, time): (Option<Res<Anchor>>, Res<Time>),
    mut flings: MessageWriter<Fling>,
) {
    let key = (time.elapsed_secs() * 1000.0) as u32;
    for (index, impact) in impacts.read().enumerate() {
        let way = anchor.as_deref().map_or(Vec3::Y, |a| up(a, impact.at));
        let salt = key.wrapping_add(index as u32 * 977);
        for spray in sprays(impact).into_iter().flatten() {
            for n in 0..spray.count {
                flings.write(Fling {
                    at: impact.at,
                    velocity: velocity(&spray, way, (n, salt)),
                    radius: spray.radius,
                    colour: spray.colour,
                    wet: false,
                    tag: None,
                });
            }
        }
    }
}
