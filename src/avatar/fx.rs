//! What changes besides the joints: the fabric of the wingsuit, the boot plumes and the face.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::Mode;
use super::animate::Anim;
use super::face::machine::Visor;
use super::face::paint::{paint, rgba};
use super::face::{Frame, ambient, gaze, landing};
use super::hurt::eye;
use super::own::Own;
use super::rig::globals;
use super::spawn::{Kit, Lamp, Plume};
use super::tune::{LANDING_FACE, LIGHT_POWER, NOZZLE, PLUME, PLUME_HDR, SCREEN_HDR};
use super::wing::{Panel, RIM, fan, flutter, rim, stretch};
use crate::condition::Condition;

/// How fast the screen colour follows the face, per second.
const TINT_RATE: f32 = 6.0;

/// Share of the full glow a nozzle keeps with no thrust.
const PILOT: f32 = 0.04;

/// Returns the length of a boot plume in meters for a thrust from 0 to 1.
pub fn plume_length(thrust: f32) -> f32 {
    PLUME.0 + (PLUME.1 - PLUME.0) * thrust.clamp(0.0, 1.0)
}

/// Returns the unlit colour of the plume and the nozzle glow for a thrust from 0 to 1.
///
/// It is linear and goes above one, so bloom makes it glow. A resting nozzle keeps a faint pilot light.
pub fn plume_color(thrust: f32) -> Color {
    let gain = PILOT + (1.0 - PILOT) * thrust.clamp(0.0, 1.0);
    Color::linear_rgb(
        PLUME_HDR[0] * gain,
        PLUME_HDR[1] * gain,
        PLUME_HDR[2] * gain,
    )
}

/// Writes the triangles of a panel into its mesh in place, with their flat normals.
fn drape(mesh: &mut Mesh, triangles: &[[Vec3; 3]; RIM]) {
    let normals = triangles.map(|[a, b, c]| (b - a).cross(c - a).normalize_or_zero().to_array());
    if let Some(VertexAttributeValues::Float32x3(points)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for (slot, corner) in points.iter_mut().zip(triangles.iter().flatten()) {
            *slot = corner.to_array();
        }
    }
    if let Some(VertexAttributeValues::Float32x3(slots)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for (slot, normal) in slots.iter_mut().zip(normals.iter().flat_map(|n| [n; 3])) {
            *slot = *normal;
        }
    }
}

/// Stretches the fabric of every body between its joints while its wingsuit is out, and hides
/// it when folded.
pub fn wings(
    bodies: Query<&Anim>,
    mut panels: Query<(&Panel, &Mesh3d, &mut Visibility, &ChildOf)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (panel, mesh, mut visibility, parent) in &mut panels {
        let Ok(body) = bodies.get(parent.parent()) else {
            continue;
        };
        let open = stretch(body.clock.unfold);
        let folded = open <= 0.0;
        visibility.set_if_neq(if folded {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        if folded {
            continue;
        }
        let wave =
            body.clock.wave(13.0, *panel as u8 as f32 * 2.1) + 0.5 * body.clock.wave(23.0, 1.0);
        let bulge = flutter(body.input.airspeed, wave / 1.5, open);
        if let Some(mut mesh) = meshes.get_mut(&mesh.0) {
            let joints = globals(&body.pose);
            drape(&mut mesh, &fan(&rim(*panel, &joints, open), bulge));
        }
    }
}

/// Returns the thrust of a body as the plumes show it: from 0 to 1, and 0 for a value that is
/// not a number.
fn pushed(body: &Anim) -> f32 {
    match body.input.thrust.is_finite() {
        true => body.input.thrust.clamp(0.0, 1.0),
        false => 0.0,
    }
}

/// Lengthens and brightens the boot plumes of every body with its thrust. They flicker a
/// little and vanish at rest.
pub fn boots(
    mut bodies: Query<(&Anim, &mut Kit)>,
    mut plumes: Query<(&Plume, &mut Transform, &mut Visibility)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (plume, mut transform, mut visibility) in &mut plumes {
        let Ok((anim, _)) = bodies.get(plume.0) else {
            continue;
        };
        let thrust = pushed(anim);
        let flicker = 1.0 + 0.08 * anim.clock.wave(47.0, 0.0);
        let width = NOZZLE.0 * (0.7 + 0.25 * thrust);
        transform.scale = Vec3::new(width, plume_length(thrust) * flicker, width);
        visibility.set_if_neq(if thrust > 0.01 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (anim, mut kit) in &mut bodies {
        let thrust = pushed(anim);
        if (thrust - kit.thrust).abs() < 1e-3 {
            continue;
        }
        kit.thrust = thrust;
        if let Some(mut material) = materials.get_mut(&kit.glow) {
            material.base_color = plume_color(thrust);
        }
    }
}

/// Repaints the screen when the frame to show is not the one painted last.
fn repaint(images: &mut Assets<Image>, kit: &mut Kit, frame: Frame) {
    if kit.shown == Some(frame) {
        return;
    }
    if let Some(mut image) = images.get_mut(&kit.image) {
        image.data = Some(rgba(&paint(&frame)));
        kit.shown = Some(frame);
    }
}

/// Runs the face of every body: notices landings, repaints the screen when the frame changes
/// and eases its colour, which also colours the faint light in front of it.
pub fn face(
    (time, own, hurt): (Res<Time>, Own, Option<Res<Condition>>),
    mut bodies: Query<(&Anim, &mut Visor, &mut Kit)>,
    mut stores: (ResMut<Assets<Image>>, ResMut<Assets<StandardMaterial>>),
    mut lamps: Query<(&Lamp, &mut PointLight)>,
) {
    let dt = time.delta_secs();
    let mine = own.body().map(|(_, body)| body);
    for (lamp, mut light) in &mut lamps {
        let Ok((anim, mut visor, mut kit)) = bodies.get_mut(lamp.0) else {
            continue;
        };
        let input = &anim.input;
        let touched = |mode: Mode| matches!(mode, Mode::Land | Mode::Roll);
        if touched(input.mode) && !touched(visor.was) {
            visor.react(landing(input.landed_hard), LANDING_FACE);
        }
        visor.was = input.mode;
        visor.advance(ambient(input, anim.clock.still), dt);
        repaint(&mut stores.0, &mut kit, visor.frame(gaze(anim.neck.look)));
        let tint = kit
            .tint
            .lerp(visor.shown().tint(), 1.0 - (-TINT_RATE * dt).exp());
        let dim = eye::screen(hurt.as_deref(), mine == Some(lamp.0), time.elapsed_secs());
        light.color = Color::linear_rgb(tint.x, tint.y, tint.z);
        light.intensity = LIGHT_POWER * dim;
        if tint.distance(kit.tint) < 1e-4 && (dim - kit.dim).abs() < 1e-3 {
            continue;
        }
        kit.tint = tint;
        kit.dim = dim;
        if let Some(mut material) = stores.1.get_mut(&kit.visor) {
            let lit = tint * SCREEN_HDR * dim;
            material.base_color = Color::linear_rgb(lit.x, lit.y, lit.z);
        }
    }
}
