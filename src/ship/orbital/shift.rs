//! The changes of regime of the orbital cruise: a cruise starts, stops at rest over the ground,
//! hands over to the glide, and the glide ends.

use bevy::math::DVec3;

use super::glide;
use super::near::Over;
use super::skim::{Band, attitude};
use super::tune::{GLIDE_ANGLE, NOTE_TIME};
use super::{Note, Orbital, Regime};
use crate::ship::body::Ship;

impl Orbital {
    /// Starts a cruise of `ship` over `over`, toward the planet point `goal` when there is one:
    /// along the way the nose points over the ground, at the height the ship is at, kept inside
    /// the band of the planet, with the lever at full.
    pub(super) fn start(&mut self, ship: &mut Ship, over: &Over, goal: Option<DVec3>) {
        let rel = ship.pos - over.centre;
        let up = rel.normalize_or(DVec3::Y);
        let band = Band::of(&over.planet);
        let ahead = ship.forward();
        self.heading = (ahead - up * ahead.dot(up)).normalize_or(up.any_orthonormal_vector());
        self.height = (rel.length() - band.radius).clamp(band.floor, band.ceiling);
        (self.regime, self.goal, self.speed) = (Regime::Cruise, goal, 0.0);
        (self.pending, self.note, self.ready) = (None, None, false);
        (ship.throttle, ship.assist, ship.spin) = (1.0, true, DVec3::ZERO);
    }

    /// Ends the cruise and leaves `ship` at rest over the ground of `over`, with `note` on the
    /// prompt: a ship that does not move against the air heats in no entry.
    pub(super) fn rest(&mut self, ship: &mut Ship, over: &Over, note: Option<Note>) {
        let rel = ship.pos - over.centre;
        ship.vel = over.vel + over.ground(rel);
        ship.throttle = 0.0;
        self.off(note);
    }

    /// Hands the cruise over to the glide: the drive leaves `ship` on the glide path at the
    /// glide speed against the ground, its nose along the path.
    pub(super) fn descend(&mut self, ship: &mut Ship, over: &Over) {
        let rel = ship.pos - over.centre;
        let up = rel.normalize_or(DVec3::Y);
        let path = self.heading * GLIDE_ANGLE.cos() - up * GLIDE_ANGLE.sin();
        let height = rel.length() - over.planet.radius;
        ship.vel = over.vel + over.ground(rel) + path * glide::speed(&over.planet, height);
        ship.rot = attitude(up, self.heading, GLIDE_ANGLE);
        self.regime = Regime::Glide;
    }

    /// Ends the cruise or the glide where the ship is, with `note` on the prompt when there is
    /// one.
    pub(super) fn off(&mut self, note: Option<Note>) {
        (self.regime, self.goal, self.reading, self.speed) = (Regime::Off, None, None, 0.0);
        self.note = note.map(|note| (note, NOTE_TIME)).or(self.note);
    }

    /// Counts down by `dt` seconds the time the last note stays on the prompt.
    pub(super) fn tick(&mut self, dt: f64) {
        self.note = self
            .note
            .map(|(note, left)| (note, left - dt))
            .filter(|(_, left)| *left > 0.0);
    }
}
