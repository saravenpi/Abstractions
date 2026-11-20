//! The height of the waves under the game camera, so the underwater view meets the surface the
//! water shader draws.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::follow::wind_in_seat;
use super::seat::Seat;
use super::wake::{SeaWakes, Wakes};
use super::waves::{WAVE_COUNT, Wave, waves_for};
use super::{Planet, Sea};
use crate::render::{GameCamera, Swell, smoothstep};
use crate::space::Anchor;
use crate::wind::Wind;

/// Gravity in metres per second squared the waves travel by, `GRAVITY` in `water.wesl`.
const GRAVITY: f32 = 9.81;

/// Phase in radians of each wave at the origin, the seeds of `waves` in `water.wesl`.
const SEEDS: [f32; WAVE_COUNT] = [0.0, 1.7, 3.1, 4.9];

/// Depth in metres of water over which the waves die out toward the shore, `SHOAL_DEPTH` in `water.wesl`.
const SHOAL_DEPTH: f32 = 2.5;

/// Returns the height in metres of `waves` and of the `wake` height on top of them above the
/// mean sea at sea coordinates `p` and `time` seconds, in water `depth` metres deep, as
/// `water.wesl` raises them.
pub fn crest(waves: &[Wave; WAVE_COUNT], (p, wake): (Vec2, f32), time: f32, depth: f32) -> f32 {
    let open: f32 = waves
        .iter()
        .zip(SEEDS)
        .map(|(wave, seed)| {
            let k = TAU / wave.wavelength;
            let phase = k * wave.direction.dot(p) - (GRAVITY * k).sqrt() * time + seed;
            wave.amplitude * phase.sin()
        })
        .sum();
    (open + wake) * smoothstep(0.0, SHOAL_DEPTH, depth)
}

/// Returns the height in metres the live `wakes` raise at sea coordinates `p` at clock `now`, in the
/// chart of `seat`, none when nothing stirs the sea.
fn stirred(wakes: Option<&Wakes>, seat: &Seat, (p, now): (Vec2, f32)) -> f32 {
    wakes
        .filter(|wakes| wakes.stirred(now))
        .map_or(0.0, |wakes| {
            SeaWakes::pack(wakes, now, |at| seat.coords(at)).height(p, now)
        })
}

/// What the swell reads of the world: the anchor, the planet and the wakes, each when it exists.
type World<'w> = (
    Option<Res<'w, Anchor>>,
    Option<Res<'w, Planet>>,
    Option<Res<'w, Wakes>>,
);

/// Writes how high the waves stand under the game camera, none where the near sea is not drawn.
pub fn swell(
    cameras: Query<&GlobalTransform, With<GameCamera>>,
    seas: Query<(&Seat, &Transform, &Visibility), With<Sea>>,
    world: World,
    (wind, time): (Res<Wind>, Res<Time>),
    mut swell: ResMut<Swell>,
) {
    let (Some(anchor), Some(planet)) = (world.0.as_deref(), world.1.as_deref()) else {
        return;
    };
    let shown = seas
        .iter()
        .find(|(seat, _, seen)| **seen != Visibility::Hidden && seat.holds(&planet.globe));
    let height = shown
        .zip(cameras.iter().next())
        .map_or(0.0, |((seat, pose, _), camera)| {
            let at = anchor.to_frame(camera.translation());
            let depth = -planet.globe.height(at.normalize());
            let waves = waves_for(&wind_in_seat(&wind, pose.rotation));
            let (p, now) = (seat.coords(at), time.elapsed_secs());
            let wake = stirred(world.2.as_deref(), seat, (p, now));
            crest(&waves, (p, wake), now, depth)
        });
    swell.set_if_neq(Swell(height));
}
