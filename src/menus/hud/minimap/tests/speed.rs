//! A fast flight low over the ground: the minimap speeds up to 300 m/s, then flies a minute at
//! that speed, and in every frame it shows a whole image that reaches past every corner of the
//! view, swaps only to whole images, and never hides its ground.

use std::time::{Duration, Instant};

use bevy::math::DVec3;
use bevy::prelude::*;

use super::read::part;
use super::rig::{app, ground, settle};
use crate::menus::hud::minimap::ahead::cover;
use crate::menus::hud::minimap::paint::BYTES;
use crate::menus::hud::minimap::patch::{Patch, TEXELS};
use crate::menus::hud::minimap::range::RANGES;
use crate::menus::hud::minimap::scope::Scope;
use crate::menus::hud::minimap::widget::Part;
use crate::render::GameCamera;
use crate::space::Anchor;

/// Ground speed of the flight in metres per second.
const SPEED: f64 = 300.0;

/// Seconds taken to speed up to [`SPEED`], as a ship does.
const RAMP: f64 = 10.0;

/// Seconds flown at [`SPEED`].
const CRUISE: f64 = 60.0;

/// Seconds of one frame of the rig.
const FRAME: f64 = 0.016;

/// Least wall time of one frame, so the game runs at most twice as fast as the workers paint.
const PACE: Duration = Duration::from_millis(8);

/// Returns the metres flown north after `seconds`: a steady speed-up, then [`SPEED`].
fn flown(seconds: f64) -> f64 {
    let ramp = seconds.min(RAMP);
    0.5 * SPEED / RAMP * ramp * ramp + SPEED * (seconds - RAMP).max(0.0)
}

/// Puts the camera at the point `north` metres north of the start, looking north, for the
/// next frame.
fn stand(app: &mut App, north: f64) {
    let (at, toward) = (ground(north, 0.0), DVec3::Y);
    let anchor = *app.world().resource::<Anchor>();
    let turned = |along: DVec3| (anchor.rotation * along).as_vec3();
    let pose = Transform::from_translation(anchor.to_render(at));
    let pose = pose.looking_to(turned(toward), turned(at.normalize()));
    let mut cameras = app
        .world_mut()
        .query_filtered::<&mut Transform, With<GameCamera>>();
    for mut camera in cameras.iter_mut(app.world_mut()) {
        *camera = pose;
    }
}

/// Returns true when the image of the ground is whole: every texel there and opaque.
fn complete(app: &App) -> bool {
    let handle = app.world().resource::<Scope>().image.clone();
    let images = app.world().resource::<Assets<Image>>();
    let data = images.get(&handle).and_then(|image| image.data.as_ref());
    data.is_some_and(|data| {
        data.len() == TEXELS * TEXELS * BYTES
            && data.chunks_exact(BYTES).all(|texel| texel[3] == u8::MAX)
    })
}

/// Returns the patch shown, the range shown and how much of a map that patch covers.
fn seen(app: &App) -> (Patch, f32, f32) {
    let scope = app.world().resource::<Scope>();
    let patch = scope.patch.expect("an image in every frame");
    let dir = scope.last.expect("on the planet").1.normalize();
    (patch, scope.radius, cover(&patch, dir))
}

/// Flies one frame to where the flight is after `seconds`, checks that the view stays on the
/// image shown, that an image swapped in is whole and that the ground shows, and returns
/// whether the image was swapped and the range shown.
fn fly(app: &mut App, seconds: f64, shown: &mut Option<Patch>) -> (bool, f32) {
    stand(app, flown(seconds));
    app.update();
    let (patch, radius, most) = seen(app);
    assert!(
        most >= radius - 1e-3,
        "{radius} m past an image of {most} m at {seconds:.2} s"
    );
    let swapped = *shown != Some(patch);
    if swapped {
        *shown = Some(patch);
        assert!(complete(app), "a partial image at {seconds:.2} s");
    }
    let ground = part(app, Part::Ground).display;
    assert_eq!(ground, Display::Flex, "no ground at {seconds:.2} s");
    (swapped, radius)
}

/// At 300 m/s for a minute the minimap image has no empty texel in any frame: the view never
/// reaches past the image shown, every image swapped in is whole, and the ground never hides.
/// The range stays wide once the flight is steady.
#[test]
fn a_fast_flight_never_shows_an_empty_texel() {
    let mut app = app("speed");
    settle(&mut app);
    let frames = ((RAMP + CRUISE) / FRAME) as usize;
    let (mut swaps, mut narrowest) = (0, f32::MAX);
    let mut shown = app.world().resource::<Scope>().patch;
    for frame in 1..=frames {
        let started = Instant::now();
        let seconds = frame as f64 * FRAME;
        let (swapped, radius) = fly(&mut app, seconds, &mut shown);
        swaps += usize::from(swapped);
        if seconds > RAMP + 10.0 {
            narrowest = narrowest.min(radius);
        }
        std::thread::sleep(PACE.saturating_sub(started.elapsed()));
    }
    println!("fast flight: {swaps} images, narrowest range {narrowest} m");
    assert!(swaps > 10, "{swaps} images");
    assert!(
        narrowest >= 0.5 * RANGES[2],
        "the range fell to {narrowest} m"
    );
}
