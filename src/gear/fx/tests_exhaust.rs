//! Tests of what the effects may lay over the ground: an exhaust that is a line at speed and
//! nothing when slow, a ribbon that cannot veil the view, and materials that only ever blend.

use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::kit::{Kit, build};
use super::ribbon::{FADE_IN, NEAR, ROWS, SPAN, Strip, fill};
use super::scene::Scene;
use super::tests_fx::scene;
use super::trail::{SPACING, Trail};
use super::trails::{EXHAUST_LIFE, EXHAUST_SPEED, Source, exhaust_heat, look};
use crate::gear::Mode;

/// Glow under which a row of the ribbon shows nothing on screen.
const DARK: f32 = 0.02;

/// Returns a scene of boots pushing fully in `mode` while the body moves at `velocity`: head
/// first when `prone`, as in flight, or else upright with the nozzles down.
fn pushing(mode: Mode, velocity: Vec3, prone: bool) -> Scene {
    let mut scene = scene(mode, velocity.length(), 1.0);
    scene.state.velocity = velocity;
    if prone {
        scene.rot = Quat::from_rotation_arc(Vec3::Y, velocity.normalize_or(Vec3::Y));
    }
    scene
}

/// Returns the exhaust ribbon of a body that flew head first at `speed` for 2 s at 30 frames
/// per second, seen from 6 m to the side.
fn exhaust(speed: f32) -> Strip {
    let velocity = Vec3::new(0.35, -0.9, 0.26).normalize() * speed;
    let heat = exhaust_heat(&pushing(Mode::Fly, velocity, true));
    let mut trail = Trail::default();
    for frame in 0..=60 {
        let at = velocity.as_dvec3() * (f64::from(frame) / 30.0);
        trail.feed(at, heat, 1.0 / 30.0);
    }
    let eye = (velocity * 2.0) + Vec3::X * 6.0;
    let mut strip = Strip::default();
    let style = look(Source::Boots, eye, None);
    fill(&trail, &style, |point| point.as_vec3(), &mut strip);
    strip
}

/// Returns the length of the lit part of `strip` and its greatest width, in metres.
fn lit(strip: &Strip) -> (f32, f32) {
    let at = |index: usize| Vec3::from(strip.positions[index]);
    let rows = (0..ROWS).filter(|row| strip.colors[row * 3 + 1][3] > DARK);
    let (first, last) = (rows.clone().min(), rows.clone().max());
    let length = first
        .zip(last)
        .map_or(0.0, |(a, b)| at(a * 3 + 1).distance(at(b * 3 + 1)));
    let width = rows.map(|row| at(row * 3).distance(at(row * 3 + 2)));
    (length, width.fold(0.0, f32::max))
}

/// Boots that brake a fall lay no trail, slow or fast: they blow ahead of the body. The ones
/// of a fast flight lay all of it, and the slowest trail still spans the rows the ribbon
/// fades in over.
#[test]
fn exhaust_trails_only_behind_a_fast_body() {
    let braked = pushing(Mode::BootFall, Vec3::new(2.0, -4.96, 1.9), false);
    assert_eq!(braked.push(), 1.0, "the flares still burn");
    assert_eq!(exhaust_heat(&braked), 0.0);
    let plunge = pushing(Mode::BootFall, Vec3::new(0.4, -45.8, 0.4), false);
    assert_eq!(exhaust_heat(&plunge), 0.0);
    let flying = |speed: f32| exhaust_heat(&pushing(Mode::Fly, Vec3::X * speed, true));
    assert_eq!(flying(EXHAUST_SPEED.0), 0.0);
    assert!((flying(37.0) - 1.0).abs() < 1.0e-5);
    assert_eq!(
        exhaust_heat(&pushing(Mode::Slide, Vec3::X * 37.0, true)),
        0.0
    );
    let heats: Vec<f32> = (0..40).map(|speed| flying(speed as f32)).collect();
    assert!(heats.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(EXHAUST_SPEED.0 * EXHAUST_LIFE / SPACING as f32 > FADE_IN + 1.0);
}

/// The fault the owner saw: at the speed of a braked fall the glow spanned two points, a slab
/// a third as wide as it was long, and at speed it widened into a sheet as it dimmed. Whatever
/// the speed, the exhaust is now nothing or a line that tapers.
#[test]
fn exhaust_is_a_line_never_a_slab() {
    for speed in [0.5, 3.0, 5.5, 9.0, 12.0] {
        let strip = exhaust(speed);
        assert!(
            (0..ROWS * 3).all(|index| strip.colors[index][3] == 0.0),
            "a trail at {speed} m/s"
        );
    }
    for speed in [14.0, 16.0, 20.0, 24.0, 37.0, 60.0, 110.0] {
        let (length, width) = lit(&exhaust(speed));
        assert!(length > 10.0 * width, "{length} by {width} m at {speed}");
        assert!(width < 0.23, "{width} m wide at {speed} m/s");
    }
    assert!(lit(&exhaust(37.0)).0 > 15.0);
    assert!(look(Source::Boots, Vec3::ZERO, None).spread < 1.0);
}

/// A trail that has only just begun ends dark at its oldest point, not on a lit edge.
#[test]
fn a_short_trail_ends_dark() {
    let mut trail = Trail::default();
    for step in 0..=4 {
        trail.feed(DVec3::X * (f64::from(step) * SPACING * 1.01), 1.0, 0.01);
    }
    let last = trail.len();
    assert!((3..6).contains(&last));
    let mut strip = Strip::default();
    let style = look(Source::Boots, Vec3::Y * 20.0, None);
    fill(&trail, &style, |point| point.as_vec3(), &mut strip);
    let middle = |row: usize| strip.colors[row * 3 + 1][3];
    assert!(middle(last - 1) > 0.3);
    assert!((last..ROWS).all(|row| middle(row) == 0.0));
}

/// The other half of the fault: a body that falls fast drags its trail up through its own
/// eyes. However near a trail passes, no row is drawn wider than [`SPAN`] of its distance to the
/// camera, and nothing glows within [`NEAR`] of it. From the side it still shows.
#[test]
fn a_trail_through_the_eye_never_veils_the_view() {
    let mut trail = Trail::default();
    for frame in 0..=30 {
        let at = DVec3::new(0.1, 30.0 - f64::from(frame), 0.0);
        trail.feed(at, 1.0, 1.0 / 30.0);
    }
    let eyes = [Vec3::Y * 1.6, Vec3::new(0.02, 9.0, 0.01), Vec3::X * 8.0];
    let mut most = [0.0_f32; 3];
    for (eye, most) in eyes.into_iter().zip(&mut most) {
        let mut strip = Strip::default();
        let style = look(Source::Boots, eye, None);
        fill(&trail, &style, |point| point.as_vec3(), &mut strip);
        for row in 0..ROWS {
            let at = |slot: usize| Vec3::from(strip.positions[row * 3 + slot]);
            let (away, glow) = (at(1).distance(eye), strip.colors[row * 3 + 1][3]);
            assert!(at(0).distance(at(2)) <= 2.0 * SPAN * away + 1.0e-5);
            assert!(away > NEAR.0 || glow == 0.0, "row {row} glows at {away} m");
            *most = most.max(glow);
        }
    }
    assert!(
        most[2] > 0.5,
        "the trail must still show from 8 m to the side"
    );
}

/// Every effect material blends over the scene: Bevy keeps such a material out of the depth,
/// normal, motion and deferred prepasses and draws it without depth write, so no effect can
/// hide the ground or change what ambient occlusion and the sea read. Flames, trails, streaks
/// and sparks only add light; dust and spray are the two that veil.
#[test]
fn effects_blend_and_never_write_depth() {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.run_system_once(build).expect("the kit builds");
    let kit = world.resource::<Kit>().clone();
    let materials = world.resource::<Assets<StandardMaterial>>();
    let light = [&kit.glow, &kit.streak, &kit.spark, &kit.flare];
    let veil = [&kit.dust, &kit.spray];
    for (handle, mode) in light
        .map(|handle| (handle, AlphaMode::Add))
        .into_iter()
        .chain(veil.map(|handle| (handle, AlphaMode::Blend)))
    {
        let material = materials.get(handle).expect("a kit material");
        assert_eq!(material.alpha_mode, mode);
        assert!(material.unlit && !material.fog_enabled);
        assert_eq!(material.depth_bias, 0.0);
    }
}
