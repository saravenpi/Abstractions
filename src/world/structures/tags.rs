//! Roles of drawn pieces, recorded in test builds so that tests can measure doors, roofs and their supports.

use super::brush::Build;

/// What a run of drawn geometry is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// Walls of a room the player can enter.
    Wall,
    /// Casing around a doorway.
    Frame,
    /// Roof slopes, slabs and awnings.
    Roof,
    /// Wall-coloured fill between the wall tops and the roof.
    Fill,
    /// A post or pole that carries a roof.
    Post,
    /// An upright prism or cone.
    Round,
    /// Leaves, cloth, straw, fish and the glass around a flame: what a body passes through, so it needs no
    /// collider.
    Soft,
    /// Streets, footpaths and plazas painted on the ground, which carries them.
    Paved,
}

impl Build<'_> {
    /// Runs `draw` and, in test builds, records the vertices it adds under `tag`.
    pub fn tagged(&mut self, tag: Tag, draw: impl FnOnce(&mut Self)) {
        #[cfg(test)]
        let start = self.vertex_mark();
        draw(self);
        #[cfg(test)]
        self.tags.push((tag, start, self.vertex_mark()));
        #[cfg(not(test))]
        let _ = tag;
    }
}
