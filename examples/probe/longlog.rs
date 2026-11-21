//! The log lines of the long play: the clock, the anchor, the camera and the scene, checked
//! for NaN, and the distinct visibility ranges handed to Bevy so far.

use bevy::camera::Exposure;
use bevy::camera::visibility::VisibilityRange;
use bevy::pbr::{DistanceFog, FogFalloff};

use super::*;

/// Logs the clock, the anchor, the camera, the scene and the ranges handed to Bevy so far.
pub fn long_report(world: &mut World, frame: u32) {
    let clock = *world.resource::<abstractions::space::Clock>();
    let anchor = *world.resource::<Anchor>();
    let ranges = world.resource::<RangeTally>().0.len();
    eprintln!(
        "PROBE long {frame}: clock {:.1} s, as f32 {:.1}, phase {:.3}, cloud drift time {:.1}, anchor {:?} \
         {:.0} m from the frame origin, {ranges} distinct visibility ranges so far",
        clock.seconds,
        clock.seconds as f32,
        clock.phase(),
        clock.seconds.rem_euclid(131_072.0),
        anchor.frame,
        anchor.origin.length(),
    );
    camera_line(world);
    scene_line(world);
}

/// What the probe reads of the game camera: its place, projection, fog and exposure.
type CameraParts = (
    &'static GlobalTransform,
    &'static Projection,
    Option<&'static DistanceFog>,
    Option<&'static Exposure>,
);

/// Logs the planes, the fog and the exposure of the game camera, with a check for NaN.
fn camera_line(world: &mut World) {
    let read = world.run_system_once(
        |cameras: Query<CameraParts, With<abstractions::render::GameCamera>>| {
            cameras
                .iter()
                .map(|(at, projection, fog, exposure)| camera_text(at, projection, fog, exposure))
                .collect::<Vec<_>>()
        },
    );
    for line in read.into_iter().flatten() {
        eprintln!("PROBE long camera: {line}");
    }
}

/// Describes a camera: how far it is from the render origin, its planes, its exposure and its fog.
fn camera_text(
    at: &GlobalTransform,
    projection: &Projection,
    fog: Option<&DistanceFog>,
    exposure: Option<&Exposure>,
) -> String {
    let planes = match projection {
        Projection::Perspective(p) => format!("near {} far {} fov {:.3}", p.near, p.far, p.fov),
        _ => "not perspective".to_string(),
    };
    let ev = exposure.map_or(f32::NAN, |e| e.ev100);
    let from = at.translation().length();
    format!(
        "{from:.1} m from the render origin, {planes}, ev100 {ev}, {}",
        fogged(fog)
    )
}

/// Describes the fog of the camera and says whether every number of it is finite.
fn fogged(fog: Option<&DistanceFog>) -> String {
    let Some(fog) = fog else {
        return "no fog".to_string();
    };
    let extinction = match &fog.falloff {
        FogFalloff::Atmospheric { extinction, .. } => *extinction,
        _ => Vec3::NAN,
    };
    let (light, color) = (
        fog.directional_light_color.to_linear(),
        fog.color.to_linear(),
    );
    let layers = Vec4::new(
        light.red,
        light.green,
        light.blue,
        fog.directional_light_exponent,
    );
    let tint = Vec4::new(color.red, color.green, color.blue, color.alpha);
    let finite = extinction.is_finite() && layers.is_finite() && tint.is_finite();
    let verdict = if finite { "finite" } else { "NAN IN FOG" };
    format!("fog extinction {extinction:?} layers {layers:?} colour {tint:?} {verdict}")
}

/// Logs the meshes in view, the farthest of them from the render origin, those at a NaN place,
/// and the visibility ranges alive now and broken ones among them.
fn scene_line(world: &mut World) {
    let read = world.run_system_once(
        |meshes: Query<(&GlobalTransform, &ViewVisibility), With<Mesh3d>>,
         ranges: Query<&VisibilityRange>| {
            let shown = || {
                meshes
                    .iter()
                    .filter(|(_, v)| v.get())
                    .map(|(at, _)| at.translation())
            };
            let far = shown().fold(0.0_f32, |far, at| far.max(at.length()));
            let lost = shown().filter(|at| !at.is_finite()).count();
            let margins = |r: &VisibilityRange| {
                [r.start_margin.start, r.start_margin.end, r.end_margin.start]
            };
            let broken = ranges
                .iter()
                .filter(|r| margins(r).iter().any(|m| m.is_nan()))
                .count();
            (shown().count(), far, lost, ranges.iter().count(), broken)
        },
    );
    if let Ok((shown, far, lost, alive, broken)) = read {
        eprintln!(
            "PROBE long scene: {shown} meshes in view, farthest {far:.0} m from the render origin, {lost} at NaN, \
             {alive} visibility ranges alive, {broken} with a NaN margin"
        );
    }
}
