//! Shared mesh building for the robot and the companion: a soup of flat triangles with one colour
//! each, and the small vocabulary that shapes it, a loft through cross-sections and a chamfered
//! block. A base area: it imports no other area of the game.

mod block;
mod loft;
mod soup;

pub use block::{Block, block};
pub use loft::{Section, drum, lathe, orb};
pub use soup::Soup;
