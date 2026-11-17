//! The trails: two wingtip ribbons and the boot exhaust.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::kit::Kit;
use super::ribbon::{Look, Strip, fill, mesh, write};
use super::scene::{SOLES, Scene};
use super::trail::Trail;
use crate::render::{Palette, smoothstep as smooth};
use crate::screens::Screen;
use crate::space::Anchor;

/// Seconds a wingtip trail glows for.
const TIP_LIFE: f32 = 1.5;

/// Seconds the exhaust glows for.
const EXHAUST_LIFE: f32 = 0.7;

/// Wingtips in the body model frame, used when no body is drawn: +Y the head, so they sit to
/// each side of the shoulders.
const TIPS: [Vec3; 2] = [Vec3::new(-0.85, 0.35, 0.0), Vec3::new(0.85, 0.35, 0.0)];

/// Linear HDR colour of the exhaust: warm orange.
const EXHAUST: Vec3 = Vec3::new(3.8, 1.3, 0.16);

/// Share of white in the wingtip colour; the rest is the sky.
const TIP_WHITE: f32 = 0.55;

/// Brightness of the wingtip colour, above 1 so that bloom catches it.
const TIP_GAIN: f32 = 2.4;

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
/// A ribbon stays at the render origin: its vertices are rewritten in render space each frame
/// from points kept in the planet frame, which is what carries it through a rebase.
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

/// Returns how strongly a wingtip glows: nothing slow or folded, faint in steady flight, bright
/// under load in pull-ups and tight turns and in fast dives, like a wingtip vortex.
pub fn tip_heat(scene: &Scene) -> f32 {
    let state = &scene.state;
    if !scene.active || !state.mode.airborne() {
        return 0.0;
    }
    let load = 0.3 + 0.7 * smooth(1.1, 2.5, state.load) + 0.5 * smooth(50.0, 85.0, state.airspeed);
    state.spread() * smooth(18.0, 40.0, state.airspeed) * scene.sigma.min(1.0) * load
}

/// Returns where `source` is in render space and how strongly it glows.
fn emitter(source: Source, scene: &Scene) -> (Vec3, f32) {
    let at = match (source, scene.limbs) {
        (Source::Tip(side), Some(limbs)) => limbs.tips[side],
        (Source::Tip(side), None) => scene.at(TIPS[side]),
        (Source::Boots, Some(limbs)) => (limbs.soles[0] + limbs.soles[1]) * 0.5,
        (Source::Boots, None) => scene.at(Vec3::NEG_Y * SOLES),
    };
    match source {
        Source::Tip(_) => (at, tip_heat(scene)),
        Source::Boots => (at, scene.push()),
    }
}

/// Returns how `source` is drawn: thin sky-tinted wingtip lines, a wider warm exhaust.
fn look(source: Source, eye: Vec3, palette: Option<&Palette>) -> Look {
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
            spread: 2.5,
            life: EXHAUST_LIFE,
            tint: EXHAUST,
        },
    }
}

/// Feeds every trail from its emitter and rewrites the meshes of the ones that glow.
///
/// A trail with nothing left to show is written once more, to clear it, and then left alone.
pub fn draw(mut canvas: Canvas, mut ribbons: Query<(&mut Ribbon, &Mesh3d)>) {
    let Some(anchor) = canvas.anchor.as_deref().filter(|_| canvas.scene.active) else {
        return;
    };
    let mut strip = Strip::default();
    for (mut ribbon, handle) in &mut ribbons {
        let (at, heat) = emitter(ribbon.source, &canvas.scene);
        ribbon
            .trail
            .feed(anchor.to_frame(at), heat, canvas.scene.dt);
        let style = look(ribbon.source, canvas.scene.eye, canvas.palette.as_deref());
        let live = ribbon.trail.live(style.life);
        if !live && !ribbon.drawn {
            continue;
        }
        ribbon.drawn = live;
        fill(
            &ribbon.trail,
            &style,
            |point| anchor.to_render(point),
            &mut strip,
        );
        if let Some(mut mesh) = canvas.meshes.get_mut(&handle.0) {
            write(&mut mesh, &strip);
        }
    }
}
