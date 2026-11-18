//! The cockpit as parts of a ship: the fixed interior, the stick, the throttle lever and the
//! lit instruments, the system that feeds the cockpit of the local ship from its helm, and
//! the system that moves the parts of every cockpit from its own state.

use bevy::light::NotShadowCaster;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;

use super::{Cockpit, Dial, LEVER, Levers, STICK};
use crate::player::Player;
use crate::ship::body::Ship;
use crate::ship::helm::Helm;
use crate::ship::mesh::{
    Skin, bar_mesh, cabin_meshes, faces_mesh, lever_mesh, stick_mesh, tub_mesh,
};
use crate::ship::telemetry::ShipTelemetry;
use crate::sim::PlayerId;
use crate::space::{Location, Travel};

/// Marker for the centre stick, a child of the ship that turns in its hinge.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Stick;

/// Marker for the throttle lever, a child of the ship that turns in its hinge.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Lever;

/// A mesh with the material it is drawn with.
type Part = (Handle<Mesh>, Handle<StandardMaterial>);

/// Returns an unlit material of one grey `level`: white for what an instrument lights, black
/// for its face. Unlit, an instrument reads the same by day and by night.
fn ink(level: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::linear_rgb(level, level, level),
        unlit: true,
        opaque_render_method: OpaqueRendererMethod::Forward,
        ..default()
    }
}

/// Returns the lining of the tub: a matte middle grey, between the dark furniture and the
/// light body of the pilot, so that both read against it.
fn lining() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(0.36, 0.38, 0.42),
        perceptual_roughness: 0.9,
        ..default()
    }
}

/// The meshes and materials of a cockpit. They are made once and shared by every ship.
#[derive(Debug, Clone)]
pub struct Kit {
    /// The fixed interior: the cabin, the tub and the faces of the instruments.
    fixed: Vec<Part>,
    /// The centre stick.
    stick: Part,
    /// The throttle lever.
    lever: Part,
    /// The lit bar of the instruments and its white.
    lit: Part,
}

impl Kit {
    /// Creates the meshes and the materials of the cockpit, with the `skins` of the ship.
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        skins: &[Handle<StandardMaterial>; 5],
    ) -> Self {
        let trim = skins[Skin::Trim as usize].clone();
        let mut fixed: Vec<Part> = cabin_meshes()
            .into_iter()
            .map(|(skin, mesh)| (meshes.add(mesh), skins[skin as usize].clone()))
            .collect();
        fixed.push((meshes.add(tub_mesh()), materials.add(lining())));
        fixed.push((meshes.add(faces_mesh()), materials.add(ink(0.0))));
        Self {
            fixed,
            stick: (meshes.add(stick_mesh()), trim.clone()),
            lever: (meshes.add(lever_mesh()), trim),
            lit: (meshes.add(bar_mesh()), materials.add(ink(1.0))),
        }
    }

    /// Returns every part of the cockpit, for the pipeline warm-up.
    pub fn parts(&self) -> impl Iterator<Item = Part> + '_ {
        let moving = [&self.stick, &self.lever, &self.lit];
        self.fixed.iter().chain(moving).cloned()
    }
}

/// Returns the bundle that draws `part`.
fn drawn((mesh, material): &Part) -> impl Bundle {
    (Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))
}

/// Adds a cockpit to the ship `entity`: its state, the cabin, the stick and the lever in
/// their hinges and the instruments on their faces. None of it has a collider.
pub fn fit(entity: &mut EntityCommands, kit: &Kit) {
    entity.insert(Cockpit::default());
    for part in &kit.fixed {
        entity.with_child(drawn(part));
    }
    entity.with_child((Stick, drawn(&kit.stick), Transform::from_translation(STICK)));
    entity.with_child((Lever, drawn(&kit.lever), Transform::from_translation(LEVER)));
    for dial in Dial::ALL {
        entity.with_child((dial, drawn(&kit.lit), dial.place(0.0), NotShadowCaster));
    }
}

/// Feeds the cockpit of the local ship: the stick is the real pitch, yaw and roll input,
/// whoever gives it, the lever the real throttle, the instruments what they measure, and the
/// pilot the local player while seated.
pub fn feed(
    (helm, telemetry): (Res<Helm>, Res<ShipTelemetry>),
    location: Res<Location>,
    players: Query<(Entity, &PlayerId), With<Player>>,
    ship: Single<(&Ship, &mut Cockpit)>,
) {
    let (ship, mut cockpit) = ship.into_inner();
    let local = players.iter().find(|(_, id)| **id == PlayerId::LOCAL);
    let fed = Cockpit {
        asked: Levers {
            stick: helm.controls.turn.as_vec3(),
            throttle: ship.throttle as f32,
        },
        read: Dial::ALL.map(|dial| dial.reading(&telemetry, ship.gear)),
        pilot: local
            .filter(|_| location.travel == Travel::InShip)
            .map(|(player, _)| player),
        ..*cockpit
    };
    cockpit.set_if_neq(fed);
}

/// The parts of a cockpit that move.
type Moving<'a> = (
    &'a ChildOf,
    &'a mut Transform,
    Option<&'a Stick>,
    Option<&'a Lever>,
    Option<&'a Dial>,
);

/// Filter of the parts of a cockpit that move.
type Hinged = Or<(With<Stick>, With<Lever>, With<Dial>)>;

/// Moves the parts of every cockpit from the state of its own ship: what is shown follows
/// what is asked and read, and the stick, the lever and the instruments follow what is shown.
pub fn work(time: Res<Time>, mut cockpits: Query<&mut Cockpit>, mut parts: Query<Moving, Hinged>) {
    let dt = time.delta_secs();
    for mut cockpit in &mut cockpits {
        let mut next = *cockpit;
        next.step(dt);
        cockpit.set_if_neq(next);
    }
    for (parent, mut transform, stick, lever, dial) in &mut parts {
        let Ok(cockpit) = cockpits.get(parent.parent()) else {
            continue;
        };
        let moved = match (stick, lever, dial) {
            (Some(_), _, _) => transform.with_rotation(cockpit.shown.stick_turn()),
            (_, Some(_), _) => transform.with_rotation(cockpit.shown.lever_turn()),
            (_, _, Some(dial)) => dial.place(cockpit.gauges[*dial as usize]),
            _ => *transform,
        };
        transform.set_if_neq(moved);
    }
}
