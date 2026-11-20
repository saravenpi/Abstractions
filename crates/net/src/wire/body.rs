//! The one shape of every body a state record carries.

use super::{Frame, Kind};

/// Most bytes of a state body the room keeps. A longer body is dropped.
pub const BODY_MAX: usize = 128;

/// A state body: what a record of one [`Kind`] carries after its head.
///
/// A body writes itself into a buffer and reads itself back from exactly the bytes it wrote.
/// The frame says how a position is packed. A body with no position ignores it.
pub trait Body: Copy + Default {
    /// The kind of record that carries this body.
    const KIND: Kind;

    /// Most bytes an encoded body takes, in any frame. It never passes [`BODY_MAX`].
    const MAX_LEN: usize;

    /// Adds the body to `out`, with its position packed for `frame`.
    fn encode(&self, frame: &Frame, out: &mut Vec<u8>);

    /// Returns the body [`Body::encode`] wrote for `frame`, or nothing when `bytes` has the
    /// wrong length or holds a number that is not finite.
    fn decode(frame: &Frame, bytes: &[u8]) -> Option<Self>;
}
