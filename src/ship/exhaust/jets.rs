//! Where every jet of a ship is and which way it blows: the two main nozzles at the tail, the
//! four lift nozzles under the belly and the fourteen reaction-control ports. The table comes
//! from the same numbers the meshes of the nozzles are made from, so a plume leaves exactly
//! from the socket that is drawn.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::tune::RCS_EXIT;
use crate::ship::mesh::{ENGINES, EXIT, LIFT, LIFT_AT, NOZZLE_Z, Port, ports};

/// The kinds of jet, in the order of the materials of a ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A main engine.
    Main,
    /// A lift nozzle under the belly.
    Lift,
    /// A reaction-control port of the attitude thrusters.
    Rcs,
}

/// One jet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jet {
    /// What kind of jet it is.
    pub kind: Kind,
    /// Where the exhaust leaves, in body axes.
    pub at: Vec3,
    /// The unit direction of the exhaust.
    pub dir: Vec3,
    /// Radius in metres of the exit of the nozzle.
    pub exit: f32,
    /// The port, for an attitude jet.
    pub port: Option<Port>,
}

/// The plume of a jet, a child of its ship, stretched along its direction by the thrust.
#[derive(Component, Debug, Clone, Copy)]
pub struct Plume(pub Jet);

/// Returns the twenty jets of a ship: the main engines, the lift nozzles, the attitude jets.
pub fn jets() -> Vec<Jet> {
    let jet = |kind, at, dir, exit| Jet {
        kind,
        at,
        dir,
        exit,
        port: None,
    };
    let mains = ENGINES.map(|e| jet(Kind::Main, Vec3::new(e.x, e.y, NOZZLE_Z), Vec3::Z, EXIT));
    let lifts = LIFT_AT.map(|at| jet(Kind::Lift, at, Vec3::NEG_Y, LIFT.1));
    let attitude = ports().map(|port| Jet {
        port: Some(port),
        ..jet(Kind::Rcs, port.at + port.out * 0.03, port.out, RCS_EXIT)
    });
    mains.into_iter().chain(lifts).chain(attitude).collect()
}

/// Adds the plumes of a ship to `entity`, folded to nothing and hidden, drawn with `mesh`.
pub fn fit(entity: &mut EntityCommands, mesh: &Handle<Mesh>) {
    for jet in jets() {
        let at = Transform::from_translation(jet.at)
            .with_rotation(Quat::from_rotation_arc(Vec3::Z, jet.dir))
            .with_scale(Vec3::ZERO);
        let plume = (Plume(jet), Mesh3d(mesh.clone()), at);
        entity.with_child((plume, Visibility::Hidden, NotShadowCaster));
    }
}
