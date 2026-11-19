//! The trails: two wingtip ribbons and the boot exhaust.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::kit::Kit;
use super::lay::redraw;
use super::ribbon::{Look, Strip, mesh};
use super::scene::{SOLES, Scene};
use super::trail::Trail;
use super::wakes::Laid;
use crate::render::{Palette, smoothstep as smooth};
use crate::screens::Screen;
use crate::space::Anchor;

/// Seconds a wingtip trail glows for.
const TIP_LIFE: f32 = 1.5;

/// Seconds the exhaust glows for.
pub const EXHAUST_LIFE: f32 = 0.7;

/// Speeds over the ground between which the exhaust starts to trail and trails fully, in
/// metres per second. A trail is laid by distance: slower than this the glow would span one or
/// two points, a slab as wide as it is long hanging over the ground where the boots braked.
pub const EXHAUST_SPEED: (f32, f32) = (12.0, 24.0);

/// Wingtips in the body model frame, used when no body is drawn: +Y the head, so they sit to
/// each side of the shoulders.
const TIPS: [Vec3; 2] = [Vec3::new(-0.85, 0.35, 0.0), Vec3::new(0.85, 0.35, 0.0)];

/// How far the exhaust must point back along the path before it trails, as the cosine of the
/// angle between the two: nothing across the path or into it, all of it from 60 degrees. Boots
/// that brake a fall blow ahead of the body, which then falls through its own trail.
pub const EXHAUST_BEHIND: (f32, f32) = (0.0, 0.5);

/// Width of the exhaust at the end of its life, as a share of its width at the nozzles. Under
/// 1, so that it tapers to a point like a flame: a trail that widened as it dimmed lay over the
/// ground as a broad see-through sheet.
pub const EXHAUST_TAPER: f32 = 0.3;

/// Linear HDR colour of the exhaust: warm orange.
const EXHAUST: Vec3 = Vec3::new(3.8, 1.3, 0.16);

/// Share of white in the wingtip colour; the rest is the sky.
const TIP_WHITE: f32 = 0.55;

/// Brightness of the wingtip colour, above 1 so that bloom catches it.
const TIP_GAIN: f32 = 2.4;

/// Airspeeds between which the wingtips start to trail and trail fully, in metres per second:
/// from under the stall of the wing to the slowest glide it holds. A band set higher left the
/// plain glide, at about 30 m/s, with a trail too faint to see.
pub const TIP_SPEED: (f32, f32) = (12.0, 24.0);

/// Glow of a wingtip in steady flight, before load and speed add theirs.
pub const TIP_STEADY: f32 = 0.6;

/// What lays a trail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A wingtip, by its index in [`TIPS`].
    Tip(usize),
    /// The boots.
    Boots,
}

/// One trail and the mesh it is drawn into.
#[derive(Component, Debug, Clone)]
pub struct Ribbon {
    /// What lays it.
    pub source: Source,
    /// Its points, in the planet frame.
    pub trail: Trail,
    /// Whether the mesh currently shows anything.
    pub drawn: bool,
}

/// What the trails are drawn from.
#[derive(SystemParam)]
pub struct Canvas<'w> {
    /// The flight of this frame.
    scene: Res<'w, Scene>,
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The colours of the sky.
    palette: Option<Res<'w, Palette>>,
    /// The ribbon meshes.
    meshes: ResMut<'w, Assets<Mesh>>,
}

/// Spawns the three ribbons when gameplay starts, unless they already exist.
///
/// A ribbon is moved to the head of its trail each frame and its vertices are rewritten around
/// it from points kept in the planet frame, which is what carries it through a rebase. It has
/// no bounds, so the renderer sorts it among the see-through things by where the entity is.
pub fn spawn(
    mut commands: Commands,
    kit: Res<Kit>,
    mut meshes: ResMut<Assets<Mesh>>,
    old: Query<(), With<Ribbon>>,
) {
    if !old.is_empty() {
        return;
    }
    for source in [Source::Tip(0), Source::Tip(1), Source::Boots] {
        commands.spawn((
            Ribbon {
                source,
                trail: Trail::default(),
                drawn: false,
            },
            Mesh3d(meshes.add(mesh())),
            MeshMaterial3d(kit.glow.clone()),
            Transform::IDENTITY,
            NoFrustumCulling,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Returns how strongly a wingtip glows: nothing slow or folded, plain in steady flight,
/// brighter under load in pull-ups and tight turns and in fast dives, like a wingtip vortex.
pub fn tip_heat(scene: &Scene) -> f32 {
    let state = &scene.state;
    if !scene.active || !state.mode.airborne() {
        return 0.0;
    }
    let load = TIP_STEADY
        + (1.0 - TIP_STEADY) * smooth(1.1, 2.5, state.load)
        + 0.5 * smooth(50.0, 85.0, state.airspeed);
    state.spread()
        * smooth(TIP_SPEED.0, TIP_SPEED.1, state.airspeed)
        * scene.sigma.min(1.0)
        * (1.0 - scene.thin)
        * load
}

/// Returns how strongly the exhaust trails: the booster push, once the body travels fast enough
/// to draw a line and leaves its exhaust behind. Boots that brake a fall or hold a slow flight
/// show their flares alone.
pub fn exhaust_heat(scene: &Scene) -> f32 {
    let velocity = scene.velocity();
    let jet = scene
        .limbs
        .map_or(scene.rot * Vec3::NEG_Y, |limbs| limbs.jet);
    let behind = -jet.dot(velocity.normalize_or_zero());
    scene.push()
        * smooth(EXHAUST_SPEED.0, EXHAUST_SPEED.1, velocity.length())
        * smooth(EXHAUST_BEHIND.0, EXHAUST_BEHIND.1, behind)
}

/// Returns where `source` is in render space and how strongly it glows.
pub fn emitter(source: Source, scene: &Scene) -> (Vec3, f32) {
    let at = match (source, scene.limbs) {
        (Source::Tip(side), Some(limbs)) => limbs.tips[side],
        (Source::Tip(side), None) => scene.at(TIPS[side]),
        (Source::Boots, Some(limbs)) => (limbs.soles[0] + limbs.soles[1]) * 0.5,
        (Source::Boots, None) => scene.at(Vec3::NEG_Y * SOLES),
    };
    match source {
        Source::Tip(_) => (at, tip_heat(scene)),
        Source::Boots => (at, exhaust_heat(scene)),
    }
}

/// Returns how `source` is drawn: thin sky-tinted wingtip lines that widen like a vortex, a
/// wider warm exhaust that tapers.
pub fn look(source: Source, eye: Vec3, palette: Option<&Palette>) -> Look {
    let sky = palette.map_or(Vec3::ONE, |palette| {
        let zenith = Color::srgb(palette.zenith.x, palette.zenith.y, palette.zenith.z).to_linear();
        Vec3::new(zenith.red, zenith.green, zenith.blue)
    });
    match source {
        Source::Tip(_) => Look {
            eye,
            width: 0.035,
            spread: 4.0,
            life: TIP_LIFE,
            tint: sky.lerp(Vec3::ONE, TIP_WHITE) * TIP_GAIN,
        },
        Source::Boots => Look {
            eye,
            width: 0.11,
            spread: EXHAUST_TAPER,
            life: EXHAUST_LIFE,
            tint: EXHAUST,
        },
    }
}

/// Feeds every trail of the player from its emitter, moves each ribbon to the head of its
/// trail and rewrites the meshes of the ones that glow. The trails of other bodies are left
/// to [`super::wakes`].
pub fn draw(
    canvas: Canvas,
    mut ribbons: Query<(&mut Ribbon, &Mesh3d, &mut Transform), Without<Laid>>,
) {
    let Canvas {
        scene,
        anchor,
        palette,
        mut meshes,
    } = canvas;
    let Some(anchor) = anchor.as_deref() else {
        return;
    };
    let mut strip = Strip::default();
    for (mut ribbon, handle, mut transform) in &mut ribbons {
        let against = (&*scene, anchor, palette.as_deref());
        redraw(
            (&mut ribbon, handle, &mut transform),
            against,
            &mut strip,
            &mut meshes,
        );
    }
}
