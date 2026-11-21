//! Knapping: the robot strikes the rock it breaks a piece off, overhead with the held tool or with
//! a punch of its fist, stooping to a low rock, and the piece comes away at the moment the blow
//! reaches the rock.

use bevy::prelude::*;

use super::super::super::AvatarPose;
use super::super::dir::Dir;
use super::super::held::{Held, Style};
use super::super::input::Robot;
use super::super::key::{Key, carry};
use super::super::layer::Layer;
use super::super::path::Path;
use super::bend::Bend;
use super::body::still;
use super::tune::{KNAP, KNAP_MEET, STEP};
use crate::craft::build::Hands;
use crate::craft::pack::{Knock, Knocked};
use crate::world::rocks::Rock;

/// A strike at a rock under way, on the root of the local body.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Knap {
    /// Seconds since it began.
    pub t: f32,
    /// The rock struck.
    pub rock: Rock,
    /// Where it is struck, in render space.
    pub at: Vec3,
    /// True once the blow reached the rock.
    pub landed: bool,
}

/// Starts a knap on the local body for each knock asked while none runs, or answers the knock at
/// once when there is no body to strike with.
pub fn start(
    mut knocks: MessageReader<Knock>,
    (robot, busy): (Robot, Query<(), With<Knap>>),
    mut commands: Commands,
    mut knocked: MessageWriter<Knocked>,
) {
    for knock in knocks.read() {
        let Some(body) = robot.body() else {
            knocked.write(Knocked {
                rock: knock.rock,
                at: knock.at,
            });
            continue;
        };
        if busy.contains(body) {
            continue;
        }
        let knap = Knap {
            t: 0.0,
            rock: knock.rock,
            at: knock.at,
            landed: false,
        };
        commands
            .entity(body)
            .insert(knap)
            .insert_if_new(Layer::default());
    }
}

/// Returns the key of a knap `t` seconds in along `path`, how far it is under way and its coil:
/// cocked over the wind-up, down to `meet` of the path through the blow, then back to carrying.
fn timeline(path: &Path, t: f32, meet: f32) -> (Key, f32, f32) {
    let [wind, blow, back] = KNAP;
    let s = |from: f32, length: f32| ((t - from) / length).clamp(0.0, 1.0);
    if t < wind {
        let k = s(0.0, wind);
        return (carry().toward(&path.at(0.0), k), k, -k);
    }
    if t < wind + blow {
        let k = s(wind, blow);
        return (path.at(meet * k), 1.0, 2.0 * meet * k - 1.0);
    }
    let k = s(wind + blow, back);
    let coil = (2.0 * meet - 1.0) * (1.0 - k);
    (path.at(meet).toward(&carry(), k), 1.0 - k, coil)
}

/// Returns the layer of a knap `t` seconds in at the point `local` of the frame of the body: an
/// overhead blow of a tool held in `tool` hands whose head meets the rock, or a punch of the fist
/// with none, turned to the rock and stooping to it with legs that carry `still` of the stoop.
pub fn struck((local, t): (Vec3, f32), tool: Option<Hands>, still: f32) -> Layer {
    let bend = Bend::of(Some(local), still);
    let (style, meet) = match tool {
        Some(_) => (Style::Chop, KNAP_MEET),
        None => (Style::Thrust, 1.0),
    };
    let (key, engage, coil) = timeline(&Path::of(style, Dir::Up), t, meet);
    let hands = tool.unwrap_or(Hands::One);
    Layer::held(&bend.turn(&key, engage), hands, [true; 2]).with_body(
        (engage, coil, hands == Hands::One),
        &bend,
        (0.5, STEP),
    )
}

/// The bodies that knap, with what they hold and how they are posed.
type Knapping<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        &'static mut Knap,
        &'static mut Layer,
        Option<&'static Held>,
        Option<&'static AvatarPose>,
    ),
>;

/// Runs every knap a frame over the strike layer: the body winds up, strikes the rock and comes
/// back; the blow reaching the rock says so once, and the knap ends.
pub fn knap(
    (time, mut commands): (Res<Time>, Commands),
    mut bodies: Knapping,
    mut knocked: MessageWriter<Knocked>,
) {
    let dt = time.delta_secs();
    for (body, placed, mut knap, mut layer, held, pose) in &mut bodies {
        knap.t += dt;
        let local = placed.affine().inverse().transform_point3(knap.at);
        let tool = held.filter(|held| held.out).map(|held| held.build.hands);
        *layer = struck((local, knap.t), tool, pose.map_or(1.0, still));
        if !knap.landed && knap.t >= KNAP[0] + KNAP[1] {
            knap.landed = true;
            knocked.write(Knocked {
                rock: knap.rock,
                at: knap.at,
            });
        }
        if knap.t >= KNAP.iter().sum::<f32>() {
            commands.entity(body).remove::<Knap>();
            *layer = Layer::default();
        }
    }
}
