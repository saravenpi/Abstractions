//! The javelin: the ground a thrown one meets and what it does there, the trail it leaves, the
//! point it is thrown at, the pose that draws it back along the aim and the whip that lets it go.

mod aim;
mod cock;
mod ground;
mod plant;
mod pose;
mod quiver;
mod soil;
mod streak;
pub mod tune;

#[cfg(test)]
mod stage;
#[cfg(test)]
mod tests_aim;
#[cfg(test)]
mod tests_flight;
#[cfg(test)]
mod tests_ground;
#[cfg(test)]
mod tests_pose;
#[cfg(test)]
mod tests_streak;

pub use aim::{cross, far, launch};
pub use ground::{Ground, Soil};
pub use plant::{Meet, lie, meet, planted};
pub use pose::{Release, draw, drawing, released};
pub use quiver::{Quiver, quiver};
pub use soil::Planet;
pub use streak::{Streak, bare, rig, trace};
