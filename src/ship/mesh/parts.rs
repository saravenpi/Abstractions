//! Every fixed part of the ship, each with the paint it is drawn in. The meshes of the ship, the
//! far mesh, the colliders and the tests are all made from this one list, so what is drawn,
//! what collides and what the tests hold together is the same.

use bevy::prelude::*;

use super::blade::{FIN, WINGLET, band, blade};
use super::dome;
use super::lames::lames;
use super::outside::{AERIAL, lamps, left_side, rung, shield, studs};
use super::ports::{nozzles, sockets};
use super::shell;
use super::struts::{MAIN, NOSE, pod};
use super::tail::{ENGINES, Motor, motor};
use super::wing::wing;
use crate::ship::solid::Solid;

/// What a part is painted with: the five skins of the ship, and the three lights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    /// Bone hull plates, toned by their vertex colours.
    Hull,
    /// The accent colour.
    Trim,
    /// Dark metal.
    Dark,
    /// Canopy glass.
    Glass,
    /// The glowing face of the engines.
    Glow,
    /// The red light on the left wing tip.
    Red,
    /// The green light on the right wing tip.
    Green,
    /// The white lights: the tail light and the landing light.
    White,
}

/// One fixed part of the ship.
#[derive(Debug, Clone)]
pub struct Part {
    /// The group the part belongs to: the parts of a group are closed shapes together. Only the
    /// tests read it.
    #[cfg(test)]
    pub group: &'static str,
    /// What it is painted with.
    pub paint: Paint,
    /// Whether the far mesh keeps it.
    pub far: bool,
    /// Its triangles.
    pub solid: Solid,
}

/// Returns a part of `group`.
fn part(_group: &'static str, paint: Paint, far: bool, solid: Solid) -> Part {
    Part {
        #[cfg(test)]
        group: _group,
        paint,
        far,
        solid,
    }
}

/// Returns the hull and what is fixed to its skin: the glass, the spine, the heat shield, the
/// fin with its stripe, the aerial, the fittings of the left flank, the studs and the sockets of
/// the jets under the belly and in the skin.
fn hull() -> Vec<Part> {
    let [body, nose] = shell::hull();
    vec![
        part("body", Paint::Hull, true, body),
        part("nose", Paint::Hull, true, nose),
        part("canopy", Paint::Glass, true, dome::glass()),
        part("spine", Paint::Hull, true, dome::spine()),
        part("shield", Paint::Hull, false, shield()),
        part("fin", Paint::Hull, true, blade(0.0, &FIN)),
        part(
            "marks",
            Paint::Trim,
            true,
            band(0.0, &FIN, (1.48, 1.66), 0.012),
        ),
        part("mast", Paint::Dark, false, blade(0.0, &AERIAL)),
        part("panel", Paint::Hull, false, left_side()),
        part("rung", Paint::Dark, false, rung()),
        part("studs", Paint::Hull, false, studs()),
        part("ports", Paint::Dark, false, sockets()),
        part("lift", Paint::Dark, false, nozzles()),
        part("pods", Paint::Hull, false, pod(0.0, &NOSE)),
    ]
}

/// Returns the wings with their fins and the pods of the main gear under them.
fn wings() -> Vec<Part> {
    let mut all = Vec::new();
    for side in [-1.0, 1.0] {
        let tip = blade(3.2 * side, &WINGLET);
        all.push(part("wing", Paint::Hull, true, wing(side)));
        all.push(part("winglet", Paint::Hull, false, tip));
        all.push(part("pods", Paint::Hull, false, pod(1.9 * side, &MAIN)));
    }
    all
}

/// Returns the two engines, each with its pod, its bell and the glowing face of its chamber, and
/// the lights.
fn engines() -> Vec<Part> {
    let mut all = Vec::new();
    for at in ENGINES {
        let Motor { pod, bell, face } = motor(at);
        all.push(part("engine", Paint::Hull, true, pod));
        all.push(part("engine", Paint::Dark, true, bell));
        all.push(part("engine", Paint::Glow, false, face));
    }
    let [red, green, white] = lamps();
    for (paint, lamp) in [
        (Paint::Red, red),
        (Paint::Green, green),
        (Paint::White, white),
    ] {
        all.push(part("lamps", paint, false, lamp));
    }
    all
}

/// Returns the plates over the airframe: the lames of the wing roots on the dark plates of the
/// frame, their triangles of the accent colour, and the bands of the engine nacelles.
fn plates() -> Vec<Part> {
    let [lames, under, marks, bands] = lames();
    vec![
        part("lames", Paint::Hull, false, lames),
        part("lames", Paint::Dark, false, under),
        part("marks", Paint::Trim, false, marks),
        part("bands", Paint::Hull, false, bands),
    ]
}

/// Returns every fixed part of the ship.
pub fn parts() -> Vec<Part> {
    [hull(), wings(), engines(), plates()].concat()
}
