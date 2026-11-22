//! 3D silhouette assembly for every crafted tool figure, derived directly from its build.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::shape_mesh::FormMesh;
use crate::craft::build::{Build, Figure, Form, mass};

/// One oriented geometric piece of a drawn 3D tool.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    /// Unit mesh shape scaled into this piece.
    pub mesh: FormMesh,
    /// Local offset along +Y from the hand grip.
    pub at: Vec3,
    /// Full extent along each local axis before rotation.
    pub size: Vec3,
    /// Local orientation around the part's centre.
    pub rot: Quat,
    /// Material index: 0 shaft, 1 head, 2 binding or guard.
    pub paint: usize,
}

/// Returns the tool figure of `build`, using the same thresholds as the 2D icon rasterizer.
pub fn figure(build: &Build) -> Figure {
    let total = mass(&build.pieces()).max(f32::EPSILON);
    let share = build.head.mass / total;
    match build.head.form {
        Form::Edge if share > 0.6 => Figure::Axe,
        Form::Edge if build.reach() < 0.5 => Figure::Knife,
        Form::Edge => Figure::Blade,
        Form::Mass if share > 0.6 => Figure::Hammer,
        Form::Mass => Figure::Club,
        Form::Point if build.counterweight.is_some() => Figure::Javelin,
        Form::Point => Figure::Spear,
    }
}

/// Returns a rod spanning `from` to `to` along +Y with diameter `width`.
fn rod(from: f32, to: f32, width: f32, paint: usize) -> Part {
    let height = (to - from).abs().max(1e-3);
    Part {
        mesh: FormMesh::Rod,
        at: Vec3::Y * ((from + to) * 0.5),
        size: Vec3::new(width, height, width),
        rot: Quat::IDENTITY,
        paint,
    }
}

/// Returns the parts of an axe: full haft, dark binding collar, poll and sideways flared bit.
fn axe(build: &Build, shift: f32) -> Vec<Part> {
    let (len, grip, bore) = (build.shaft.length, build.grip, build.shaft.diameter);
    let size = build.head.size();
    let bit_h = (size * 1.25).max(bore * 4.0);
    let bit_x = (size * 1.15).max(bore * 4.5);
    let thick = (size * 0.16).max(bore * 1.2).max(0.025);
    let top = (len + bit_h * 0.34).min(build.reach());
    let neck = shift + len;
    let poll_w = bore * 1.6;
    let neck_x = bore * 0.45;
    vec![
        rod(shift - grip, shift + top - 0.01, bore, 0),
        rod(neck - bit_h * 0.35, neck - bit_h * 0.18, bore * 1.45, 2),
        Part {
            mesh: FormMesh::Cube,
            at: Vec3::new(-poll_w * 0.25, neck, 0.0),
            size: Vec3::new(poll_w, bit_h * 0.48, thick * 1.15),
            rot: Quat::IDENTITY,
            paint: 1,
        },
        Part {
            mesh: FormMesh::AxeBit,
            at: Vec3::new(neck_x + bit_x * 0.5, neck - bit_h * 0.06, 0.0),
            size: Vec3::new(bit_x, bit_h, thick),
            rot: Quat::IDENTITY,
            paint: 1,
        },
    ]
}

/// Returns the parts of a hammer: haft, binding collar, socket, heavy poll (+X) and peen (-X).
fn hammer(build: &Build, shift: f32) -> Vec<Part> {
    let (len, grip, bore) = (build.shaft.length, build.grip, build.shaft.diameter);
    let size = build.head.size();
    let span = (size * 1.75).max(bore * 6.0);
    let height = (size * 0.68).max(bore * 2.2);
    let depth = (size * 0.65).max(bore * 2.0);
    let poll_w = span * 0.36;
    let neck = shift + len;
    let top = (len + height * 0.55).min(build.reach());
    vec![
        rod(shift - grip, shift + top - 0.005, bore, 0),
        rod(neck - height * 0.55, neck - height * 0.32, bore * 1.45, 2),
        Part {
            mesh: FormMesh::Cube,
            at: Vec3::Y * neck,
            size: Vec3::new(span * 0.28, height * 0.9, depth * 0.9),
            rot: Quat::IDENTITY,
            paint: 1,
        },
        Part {
            mesh: FormMesh::Cube,
            at: Vec3::new(span * 0.14 + poll_w * 0.5, neck, 0.0),
            size: Vec3::new(poll_w, height, depth),
            rot: Quat::IDENTITY,
            paint: 1,
        },
        Part {
            mesh: FormMesh::Wedge,
            at: Vec3::new(-(span * 0.14 + poll_w * 0.5), neck, 0.0),
            size: Vec3::new(poll_w, height * 0.78, depth * 0.85),
            rot: Quat::from_rotation_y(PI),
            paint: 1,
        },
    ]
}

/// Returns a short-handled sword (`sword = true`) or single-edged clip-point knife reaching `reach`.
fn edged(build: &Build, shift: f32, sword: bool) -> Vec<Part> {
    let (len, grip, bore) = (build.shaft.length, build.grip, build.shaft.diameter);
    let (size, reach) = (build.head.size(), build.reach());
    let (guard_y, width, tip_share, guard_w, blade_mesh) = if sword {
        let w = (size * 0.42).clamp(0.045, 0.095);
        (
            (len * 0.14).clamp(0.05, 0.12),
            w,
            0.22,
            (w * 2.8).max(0.14),
            FormMesh::Blade,
        )
    } else {
        let w = (size * 0.45).clamp(0.035, 0.075);
        (
            (len * 0.24).clamp(0.03, 0.08),
            w,
            0.28,
            w * 1.35,
            FormMesh::Wedge,
        )
    };
    let thick = (size * 0.12).clamp(0.014, 0.028);
    let span = (reach - guard_y).max(0.08);
    let body_end = guard_y + span * (1.0 - tip_share);
    let pommel_w = build.counterweight.map_or(bore * 1.4, |w| {
        (w.mass / w.material.density).cbrt().max(bore * 1.4)
    });
    let base = shift - grip;
    vec![
        rod(base, shift + guard_y, bore * 1.05, 0),
        rod(base - bore * 0.6, base + bore * 0.4, pommel_w, 2),
        Part {
            mesh: FormMesh::Cube,
            at: Vec3::Y * (guard_y + shift),
            size: Vec3::new(guard_w, if sword { 0.024 } else { 0.018 }, bore * 1.25),
            rot: Quat::IDENTITY,
            paint: 2,
        },
        Part {
            mesh: blade_mesh,
            at: Vec3::Y * ((guard_y + body_end) * 0.5 + shift),
            size: Vec3::new(width, body_end - guard_y, thick),
            rot: Quat::IDENTITY,
            paint: 1,
        },
        Part {
            mesh: FormMesh::Point,
            at: Vec3::Y * ((body_end + reach) * 0.5 + shift),
            size: Vec3::new(width, reach - body_end, thick),
            rot: Quat::IDENTITY,
            paint: 1,
        },
    ]
}

/// Returns the parts of a spear or lighter thrown javelin reaching `build.reach()`.
fn spear(build: &Build, shift: f32, javelin: bool) -> Vec<Part> {
    let (len, grip, bore) = (build.shaft.length, build.grip, build.shaft.diameter);
    let (size, reach) = (build.head.size(), build.reach());
    let base = (len - size * 0.2).max(len * 0.5);
    let mid = base + (reach - base) * 0.34;
    let (s, b) = if javelin { (0.22, 1.6) } else { (0.30, 2.1) };
    let width = (size * s).max(bore * b);
    let thick = (width * 0.45).max(0.014);
    vec![
        rod(shift - grip, shift + base + 0.02, bore, 0),
        rod(shift + base - 0.03, shift + base + 0.02, bore * 1.4, 2),
        Part {
            mesh: FormMesh::Blade,
            at: Vec3::Y * ((base + mid) * 0.5 + shift),
            size: Vec3::new(width, mid - base, thick),
            rot: Quat::IDENTITY,
            paint: 1,
        },
        Part {
            mesh: FormMesh::Point,
            at: Vec3::Y * ((mid + reach) * 0.5 + shift),
            size: Vec3::new(width, reach - mid, thick),
            rot: Quat::IDENTITY,
            paint: 1,
        },
    ]
}

/// Returns the parts of a flared heavy club reaching `build.reach()`.
fn club(build: &Build, shift: f32) -> Vec<Part> {
    let (len, grip, bore) = (build.shaft.length, build.grip, build.shaft.diameter);
    let (size, reach) = (build.head.size(), build.reach());
    let head_start = (len - size * 0.35).max(len * 0.4);
    let head_w = (size * 0.72).max(bore * 2.1);
    let neck_h = ((head_start - shift.min(0.0)) * 0.1).max(0.03);
    let collar = shift + head_start;
    vec![
        rod(shift - grip, collar + 0.01, bore, 0),
        rod(collar - neck_h, collar, bore * 1.5, 2),
        rod(collar, shift + reach, head_w, 1),
    ]
}

/// Returns every 3D part of `build`, shifted by `shift` along +Y so `spears` and held tools
/// align at the tip or grip.
pub fn parts(build: &Build, shift: f32) -> Vec<Part> {
    let kind = figure(build);
    let mut out = match kind {
        Figure::Axe => axe(build, shift),
        Figure::Hammer => hammer(build, shift),
        Figure::Blade => edged(build, shift, true),
        Figure::Knife => edged(build, shift, false),
        Figure::Spear => spear(build, shift, false),
        Figure::Javelin => spear(build, shift, true),
        Figure::Club => club(build, shift),
    };
    if let Some(weight) = build.counterweight.filter(|_| kind != Figure::Blade) {
        let side = (weight.mass / weight.material.density).cbrt();
        out.push(Part {
            mesh: FormMesh::Cube,
            at: Vec3::Y * (shift - build.grip),
            size: Vec3::splat(side),
            rot: Quat::IDENTITY,
            paint: 1,
        });
    }
    out
}
