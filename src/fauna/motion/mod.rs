//! Procedural motion on rigid parts: springs, gaits, feet, limb solvers and chains.
//!
//! Pure: plain data and `bevy::math`, no ECS. The runtime keeps one [`Motion`] per individual,
//! steps it with a [`Drive`] and the ground height, and copies [`Motion::pose`] onto the parts.

pub mod chain;
pub mod feet;
pub mod frame;
pub mod gait;
pub mod idle;
pub mod ik;
pub mod lag;
pub mod look;
pub mod plans;
pub mod pose;
pub mod rest;
pub mod rig;
pub mod root;
pub mod settle;
pub mod spring;
pub mod stance;
pub mod strand;
pub mod tune;
pub mod turn;
pub mod walk;

pub use pose::Placed;
pub use rig::{Detail, Drive, Motion};

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod fixtures_legs;
#[cfg(test)]
mod fixtures_life;
#[cfg(test)]
mod fixtures_soft;
#[cfg(test)]
mod tests_bodies;
#[cfg(test)]
mod tests_chain;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_gait;
#[cfg(test)]
mod tests_hill;
#[cfg(test)]
mod tests_ik;
#[cfg(test)]
mod tests_plans;
#[cfg(test)]
mod tests_size;
#[cfg(test)]
mod tests_spring;
#[cfg(test)]
mod tests_walk;
