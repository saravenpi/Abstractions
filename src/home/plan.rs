//! The finished plan of the base: its meshes and everything the spawner and the tests need.

use bevy::prelude::*;

use super::draft::{Block, Draft};
use super::glow::Glow;
use super::lot::Lot;
use super::{cables, container, pad, path, station};

/// The base, built once for a world: all in the frame of the base.
pub struct Plan {
    /// Mesh of everything lit by the sun and the lamps.
    pub lit: Mesh,
    /// Meshes of the glowing parts, one per [`Glow`].
    pub glows: [Mesh; 6],
    /// Boxes nothing walks through.
    pub blocks: Vec<Block>,
    /// Where each light that is cast sits.
    pub lamps: Vec<(Glow, Vec3)>,
}

/// Draws every piece of the base on `lot`.
pub fn draw(lot: &Lot) -> Draft<'_> {
    let mut draft = Draft::new(lot);
    container::build(&mut draft);
    station::build(&mut draft);
    let socket = container::pose(lot).transform_point3(container::SOCKET);
    let plug = station::pose(lot).transform_point3(station::PLUG);
    cables::build(&mut draft, socket, plug);
    pad::build(&mut draft);
    path::build(&mut draft);
    draft
}

/// Draws the whole base on `lot` and turns it into meshes. Heavy enough to stay off the main
/// thread.
pub fn plan(lot: &Lot) -> Plan {
    let draft = draw(lot);
    Plan {
        lit: draft.lit.into_mesh(),
        glows: draft.glows.map(|soup| soup.into_mesh()),
        blocks: draft.blocks,
        lamps: draft.lamps,
    }
}
