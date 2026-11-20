//! Fitting a cockpit to a ship: the fixed interior, the controls in their hinges, the bars,
//! the lamps and the displays, each a child of the ship.

use bevy::camera::visibility::VisibilityRange;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::canopy::Canopy;
use super::kit::{Drawn, Kit};
use super::part::Part;
use super::readings::Lamp;
use super::screens::Screen;
use super::state::Cockpit;
use super::tune::{DRAW, GEAR_LEVER, HINGE, LEVER, PEDAL, STICK, STRAP, WARP_SWITCH};

/// Marks an instrument that glows: it is dimmed by day to the share of this linear tone.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lit(pub Vec3);

/// Returns the range the cockpit is drawn in: from the seat to [`DRAW`] metres from the camera.
fn range() -> VisibilityRange {
    VisibilityRange {
        start_margin: 0.0..0.0,
        end_margin: DRAW,
        use_aabb: false,
    }
}

/// Returns the bundle that draws `part`: no shadow, and only near the camera.
fn drawn((mesh, material): &Drawn) -> impl Bundle {
    (
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        NotShadowCaster,
        range(),
    )
}

/// Adds the canopy to the ship `entity`: the glass and its frame, hinged at the back, shut.
fn canopy(entity: &mut EntityCommands, kit: &Kit) {
    let hinge = (
        Canopy::default(),
        Transform::from_translation(HINGE),
        Visibility::default(),
    );
    entity.with_children(|parent| {
        let mut hinge = parent.spawn(hinge);
        for shape in &kit.canopy {
            hinge.with_child(drawn(shape));
        }
    });
}

/// Adds the controls to the ship `entity`, each in its hinge.
fn controls(entity: &mut EntityCommands, kit: &Kit) {
    let hinges = [(Part::Stick, STICK), (Part::Lever, LEVER)];
    let hinges = hinges
        .into_iter()
        .chain([(Part::Gear, GEAR_LEVER), (Part::Switch, WARP_SWITCH)]);
    for ((part, at), shape) in hinges.zip(&kit.levers) {
        entity.with_child((part, drawn(shape), Transform::from_translation(at)));
    }
    for side in 0..2 {
        let at = Vec3::new(PEDAL.x * (side as f32 * 2.0 - 1.0), PEDAL.y, PEDAL.z);
        let pedal = (
            Part::Pedal(side),
            drawn(&kit.pedal),
            Transform::from_translation(at),
        );
        entity.with_child(pedal);
    }
    let strap = (
        Part::Strap,
        drawn(&kit.strap),
        Transform::from_translation(STRAP.0),
    );
    entity.with_child(strap);
}

/// Adds the lamps and the displays to the ship `entity`, dark until a reading
/// lights them.
fn instruments(entity: &mut EntityCommands, kit: &Kit) {
    for lamp in Lamp::ALL {
        let lit = (kit.screen.clone(), kit.glass(lamp));
        entity.with_child((
            Part::Lamp(lamp),
            Lit(lamp.tone()),
            drawn(&lit),
            lamp.place(false),
        ));
    }
    for screen in Screen::ALL {
        let shape = (Mesh3d(kit.screen.clone()), NotShadowCaster, range());
        entity.with_child((screen, shape, screen.placed()));
    }
}

/// Adds a cockpit to the ship `entity`: its state, the cabin, the canopy, the controls in
/// their hinges and the instruments on their faces. None of it has a collider.
pub fn fit(entity: &mut EntityCommands, kit: &Kit) {
    entity.insert(Cockpit::default());
    for part in &kit.fixed {
        entity.with_child(drawn(part));
    }
    canopy(entity, kit);
    controls(entity, kit);
    instruments(entity, kit);
}
