//! Shared mesh building for the robot and the companion: a soup of flat triangles with one colour
//! each, and the small vocabulary that shapes it: a loft through cross-sections, a chamfered
//! plate, a drum and a ball, and the level of detail they are built at. A base area: it imports
//! no other area of the game.

mod block;
mod detail;
mod loft;
mod plate;
mod round;
mod soup;

#[cfg(test)]
pub mod rig;
#[cfg(test)]
mod tests;

pub use block::{Block, block};
pub use detail::{Detail, LARGE, SMALL};
pub use loft::{Ring, Section, loft};
pub use plate::{Plate, plates};
pub use round::{Disc, discs, orb};
pub use soup::Soup;

/// An sRGB colour.
pub type Rgb = [f32; 3];
