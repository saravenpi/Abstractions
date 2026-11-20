//! What keeps the warp drive from taking a course, and what the route says about it.
//!
//! The same checks lock the drive in `warp::request` and hold the autopilot: the ship must be
//! off the ground, out of the frame of a planet surface and outside every lock sphere.

use std::f64::consts::PI;

use bevy::math::DVec3;

use crate::computer::{Block, RouteStatus};
use crate::ship::Ship;
use crate::ship::pilot::Phase;
use crate::ship::wells::{STAR_LOCK, Well, nearest};
use crate::space::Frame;

/// Returns the first check that keeps the drive of `ship` locked among `wells`, or nothing
/// when the drive may spool.
pub fn blocker(ship: &Ship, wells: &[Well]) -> Option<Block> {
    if ship.landed {
        return Some(Block::Landed);
    }
    if matches!(ship.frame, Frame::Surface(_)) {
        return Some(Block::Surface);
    }
    let well = nearest(wells, ship.pos).filter(|well| well.clearance(ship.pos) <= 0.0)?;
    let radii = well.lock / well.radius;
    Some(Block::Inside {
        star: (radii - STAR_LOCK).abs() < 1e-6,
        lock: (radii * 10.0).round() as u8,
        tenths: (well.pos.distance(ship.pos) / well.radius * 10.0) as u32,
    })
}

/// Returns how far the nose `nose` has turned onto the course `course`, in percent: 100 on
/// the course, 0 facing away from it.
pub fn aligned(nose: DVec3, course: DVec3) -> u8 {
    let angle = nose.dot(course.normalize_or_zero()).clamp(-1.0, 1.0).acos();
    ((1.0 - angle / PI) * 100.0).floor() as u8
}

/// What the route reads to say how far along it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    /// What the autopilot does.
    pub phase: Phase,
    /// The first check that locks the drive, if any.
    pub block: Option<Block>,
    /// Percent of the turn onto the course.
    pub aligned: u8,
    /// Whether a course is set.
    pub course: bool,
}

/// Returns the state of the route the ship shows. A course that is set but not flown says
/// what keeps the drive locked, as does a route that waits for the drive. A route that
/// arrived and was given a new course since reads as idle.
pub fn status(reading: Reading) -> RouteStatus {
    match (reading.phase, reading.block) {
        (Phase::Idle, Some(block)) if reading.course => RouteStatus::Blocked(block),
        (Phase::Idle, _) => RouteStatus::Idle,
        (Phase::Waiting, Some(block)) => RouteStatus::Blocked(block),
        (Phase::Waiting, None) | (Phase::Aligning, _) => RouteStatus::Aligning(reading.aligned),
        (Phase::Climbing, _) => RouteStatus::Climbing,
        (Phase::Dropping, _) => RouteStatus::Dropping,
        (Phase::Spooling, _) => RouteStatus::Spooling,
        (Phase::Cruising, _) => RouteStatus::InWarp,
        (Phase::Arrived, _) => RouteStatus::Arrived,
    }
}
