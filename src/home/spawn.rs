//! Raising the base when the player is near it on its planet, and taking it down when not.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::base::Base;
use super::draft::Block;
use super::glow::Glow;
use super::tune::SIGHT;
use super::works::{Kit, Works};
use crate::screens::Screen;
use crate::space::{Anchor, Clock, Placed};
use crate::universe::Universe;
use crate::world::{Link, Planet, Sited, SpawnPoint, StreamAnchor};

/// Marks the root of the spawned base and carries how much the operator has noticed the robot.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Home {
    /// From 0, nobody at the hatch, to 1.
    pub notice: f32,
}

/// Marks the point light a glow casts.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lamp(pub Glow);

/// Reads where the focus of the world is, seen from the planet of the base.
#[derive(SystemParam)]
pub struct Sight<'w, 's> {
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The streamed planet.
    planet: Option<Res<'w, Planet>>,
    /// The universe, for the orbit of the planet.
    universe: Option<Res<'w, Universe>>,
    /// Universe time.
    clock: Option<Res<'w, Clock>>,
    /// The entity the world streams around.
    focus: Query<'w, 's, &'static Transform, With<StreamAnchor>>,
    /// Where the world streams around before that entity exists.
    spawn: Option<Res<'w, SpawnPoint>>,
}

impl Sight<'_, '_> {
    /// Returns the anchor and the link from the planet of `base` to the anchor frame, or nothing
    /// while another planet is streamed or the anchor is out of its system.
    pub fn link(&self, base: &Base) -> Option<(Anchor, Link)> {
        let (anchor, planet) = (self.anchor.as_deref()?, self.planet.as_deref()?);
        let time = self.clock.as_ref().map_or(0.0, |clock| clock.seconds);
        let link = Link::between(anchor.frame, planet, self.universe.as_deref()?, time)?;
        (planet.body == base.body).then_some((*anchor, link))
    }

    /// Returns the distance in metres from the focus of the world to the point `to` of the
    /// planet of `base`, or nothing where [`Sight::link`] gives nothing.
    pub fn distance(&self, base: &Base, to: bevy::math::DVec3) -> Option<f64> {
        let (anchor, link) = self.link(base)?;
        let focus = self.focus.iter().next().map(|at| at.translation);
        let eye = focus.or(self.spawn.as_ref().map(|spawn| spawn.0))?;
        Some(link.to_planet(anchor.to_frame(eye)).distance(to))
    }
}

/// Returns the solid boxes `blocks` as one collider in the frame of the base.
pub(super) fn solid(blocks: &[Block]) -> Collider {
    let shape = |block: &Block| {
        let size = block.size.max(Vec3::splat(0.02));
        (
            block.at,
            block.rot,
            Collider::cuboid(size.x, size.y, size.z),
        )
    };
    Collider::compound(blocks.iter().map(shape).collect())
}

/// Returns the root of the base: its pose on the planet, in the anchor frame and on screen, and
/// its solid boxes as one static collider.
fn root(base: &Base, kit: &Kit, anchor: &Anchor, link: &Link) -> impl Bundle {
    let sited = Sited {
        pos: base.centre,
        rot: base.rot(),
    };
    let placed = Placed {
        pos: link.to_anchor(sited.pos),
        rot: link.turn * sited.rot,
    };
    let transform = Transform {
        translation: anchor.to_render(placed.pos),
        rotation: anchor.rotation_to_render(placed.rot),
        scale: Vec3::ONE,
    };
    (
        Name::new("Home base"),
        Home::default(),
        (sited, placed, transform, Visibility::default()),
        (RigidBody::Static, solid(&kit.blocks)),
        DespawnOnEnter(Screen::Title),
    )
}

/// Spawns the meshes and the lights of `kit` as children of the root.
fn dress(root: &mut ChildSpawnerCommands, kit: &Kit) {
    root.spawn((Mesh3d(kit.lit.clone()), MeshMaterial3d(kit.paint.clone())));
    for (glow, (mesh, material)) in Glow::ALL.into_iter().zip(&kit.glows) {
        let part = (Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()));
        root.spawn((glow, part, NotShadowCaster));
    }
    for (glow, at) in &kit.lamps {
        let range = glow.lamp().map_or(1.0, |(_, range)| range);
        let light = PointLight {
            color: glow.cast(),
            intensity: 0.0,
            range,
            radius: 0.1,
            shadow_maps_enabled: false,
            ..default()
        };
        root.spawn((Lamp(*glow), light, Transform::from_translation(*at)));
    }
}

/// Spawns the base once its parts exist and the focus comes within sight on its planet, and
/// despawns it when the focus leaves farther than it came or another planet is streamed.
pub fn tend(
    mut commands: Commands,
    base: Res<Base>,
    works: Res<Works>,
    sight: Sight,
    homes: Query<Entity, With<Home>>,
) {
    let distance = sight.distance(&base, base.centre);
    match (homes.iter().next(), distance, &works.kit) {
        (Some(home), far, _) if far.is_none_or(|d| d > SIGHT.1) => {
            commands.entity(home).despawn();
        }
        (None, Some(near), Some(kit)) if near < SIGHT.0 => {
            if let Some((anchor, link)) = sight.link(&base) {
                let root = root(&base, kit, &anchor, &link);
                commands.spawn(root).with_children(|root| dress(root, kit));
            }
        }
        _ => {}
    }
}
