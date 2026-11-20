//! Blood: what carries oxygen in a body, the colour it gives the fluid, and the physics of a fluid
//! that leaves a wound.
//!
//! The carrier follows the biochemistry of the planet ([`carrier`]), so every animal of one planet
//! bleeds in related colours, and a body with no circulation bleeds nothing. The pressure, the
//! volume, the flow through an opening and the clotting follow from the mass and the height of the
//! animal and the gravity of its planet ([`flow`]). Pure functions of plain data.

mod beat;
mod carrier;
mod drops;
mod flow;
pub mod tune;

#[cfg(test)]
mod tests_carrier;
#[cfg(test)]
mod tests_flow;

pub use beat::{heart_rate, pulse};
pub use carrier::{Blood, Carrier, of, species};
pub use drops::{drag, drop_radius, drop_volume, stain};
pub use flow::{clot, gush, pressure, seep, speed, vessel, volume};
