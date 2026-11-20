//! Blood: what carries oxygen in a body, the colour it gives the fluid, and the physics of a fluid
//! that leaves a wound.
//!
//! The carrier follows the biochemistry of the planet ([`carrier`]), so every animal of one planet
//! bleeds in related colours, and a body with no circulation bleeds nothing. The pressure, the
//! volume, the flow through an opening and the clotting follow from the mass and the height of the
//! animal and the gravity of its planet ([`flow`]). Pure functions of plain data.

mod carrier;
mod flow;
pub mod tune;

#[cfg(test)]
mod tests_carrier;
#[cfg(test)]
mod tests_flow;

pub use carrier::{Blood, Carrier, of, species};
pub use flow::{
    clot, drag, drop_radius, gush, heart_rate, pressure, pulse, seep, speed, stain, vessel, volume,
};
