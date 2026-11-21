//! Where a one-shot sound is for the listener, worked out once when it starts: how loud it arrives
//! through the air and through the body, from which bearing, how late, and how dull.
//!
//! The pressure of a source falls as `1 / r` past its reach and grows with the density of the air
//! it pushes; the air takes its high end as ISO 9613-1 says; the delay is the distance over the
//! speed of sound of that air. A vacuum carries nothing but what the body touches. Under water
//! sound is faster, spreads over a cylinder in a shallow sea, and an ear there hears it dull and
//! from nowhere in particular; across the surface little gets through.

use bevy::prelude::*;

use super::air::{Absorb, Air};
use super::ear::around;
use super::listen::Listener;
use super::tune::{
    CORNERS, CROSS, DENSE_CAP, FLOOR, HIDDEN, HULL, SEA_DEPTH, WATER_SOUND, WET_EAR,
};
use crate::audio::cue::AudioCue;

/// Where a one-shot sound starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Origin {
    /// Nowhere in the world: the interface, and the cabin around a seated pilot. Heard the same
    /// in both ears, at its level.
    Flat,
    /// A point of the world in render space, and the reach of its source in metres.
    At(Vec3, f32),
    /// A point the listener's body touches, its reach, and the share of it the body carries to
    /// the ears whatever the air: the listener's own feet, the tool in its hands, a blow it takes.
    Held(Vec3, f32, f32),
    /// A shock wave from a point, with its reach, that reaches the listener now: placed and dulled
    /// but not late, since the moment it arrives is the moment it is sent.
    Arrived(Vec3, f32),
}

/// A sound of the world: what it is and where it starts. Write it with `MessageWriter<Sound>`.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Sound {
    /// The sound.
    pub cue: AudioCue,
    /// Where it starts.
    pub from: Origin,
}

/// What lies between a source and the listener, besides the air.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Between {
    /// Whether the source is under water.
    pub wet: bool,
    /// Whether the ground hides the source from the listener.
    pub hidden: bool,
}

/// How a one-shot sound reaches the two ears. It crosses to the audio thread with its cue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Gain of what the air carries, from 0.
    pub air: f32,
    /// Gain of what the body carries, from 0: nothing for a source it does not touch.
    pub body: f32,
    /// Sine of the bearing: -1 fully left, 1 fully right.
    pub side: f32,
    /// Cosine of the bearing: 1 straight ahead, -1 straight behind.
    pub front: f32,
    /// Seconds the sound takes to arrive.
    pub delay: f32,
    /// Corner in hertz of the low-pass of the way.
    pub corner: f32,
    /// Seconds the far ear hears it after the near one.
    pub gap: f32,
    /// Share of it the cabin around ears in the seat sends back as reflections.
    pub room: f32,
}

/// Number of fields of a [`Spot`].
pub const SPOT: usize = 8;

impl Spot {
    /// A sound from nowhere in the world: both ears alike, at its level, at once.
    pub const FLAT: Spot = Spot {
        air: 1.0,
        body: 0.0,
        side: 0.0,
        front: 1.0,
        delay: 0.0,
        corner: f32::INFINITY,
        gap: 0.0,
        room: 0.0,
    };

    /// Returns true when the sound arrives over the floor of hearing.
    pub fn heard(&self) -> bool {
        self.air + self.body >= FLOOR
    }

    /// Returns the fields in order, to cross to the audio thread.
    pub fn to_array(self) -> [f32; SPOT] {
        [
            self.air,
            self.body,
            self.side,
            self.front,
            self.delay,
            self.corner,
            self.gap,
            self.room,
        ]
    }

    /// Returns the spot whose fields [`Spot::to_array`] gave.
    pub fn from_array([air, body, side, front, delay, corner, gap, room]: [f32; SPOT]) -> Self {
        Self {
            air,
            body,
            side,
            front,
            delay,
            corner,
            gap,
            room,
        }
    }
}

/// The medium a sound crosses on its way to the ears.
struct Way {
    /// Speed of travel in metres per second.
    sound: f32,
    /// Speed of sound at the ears, which sets how late the far ear is.
    ear: f32,
    /// Pressure factor of the medium, and of a crossing of the water surface.
    gain: f32,
    /// Corner in hertz the crossing and a wet ear leave.
    corner: f32,
    /// Share of the bearing the ears keep.
    width: f32,
    /// Whether the sound spreads as in a sea.
    sea: bool,
    /// The absorption of the air on the way.
    absorb: Absorb,
}

impl Way {
    /// Returns the way through `air` for an ear and a source each wet or dry.
    fn of(air: &Air, (ear_wet, source_wet): (bool, bool)) -> Self {
        let (ear, width, heard) = match ear_wet {
            true => (WATER_SOUND, WET_EAR.1, WET_EAR.0),
            false => (air.sound, 1.0, f32::INFINITY),
        };
        let pushed = air.density.min(DENSE_CAP);
        let (sound, gain, corner, absorb) = match (ear_wet, source_wet) {
            (true, true) => (WATER_SOUND, 1.0, heard, Absorb::NONE),
            (false, false) => (air.sound, pushed, heard, air.absorb),
            _ => (air.sound, pushed * CROSS.0, heard.min(CROSS.1), air.absorb),
        };
        let sea = ear_wet && source_wet;
        Self {
            sound,
            ear,
            gain,
            corner,
            width,
            sea,
            absorb,
        }
    }

    /// Returns the gain at `metres` of a source of `reach`: `1 / r` past the reach, and in a sea
    /// `1 / sqrt(r)` past its depth.
    fn level(&self, metres: f32, reach: f32) -> f32 {
        let near = reach.max(1.0e-3);
        let edge = SEA_DEPTH.max(near);
        match self.sea && metres > edge {
            true => self.gain * near / edge * (edge / metres).sqrt(),
            false => self.gain * near / metres.max(near),
        }
    }

    /// Returns the corner of the low-pass of `metres` of this way.
    fn corner(&self, metres: f32) -> f32 {
        self.absorb.corner(metres).min(self.corner)
    }
}

/// Returns the share of the pressure and the corner what lies between leaves of a source
/// `metres` from ears `in_cabin` or not: a hill, and the closed hull of the ship around ears in
/// its seat for anything outside the cabin.
fn walls(between: Between, in_cabin: bool, metres: f32) -> (f32, f32) {
    let (hill, dull) = match between.hidden {
        true => HIDDEN,
        false => (1.0, CORNERS.1),
    };
    match in_cabin && metres > HULL.0 {
        true => (hill * HULL.1, dull.min(HULL.2)),
        false => (hill, dull),
    }
}

/// Returns how a sound from `from` reaches the listener `ear`, past what lies `between`.
pub fn spot(ear: &Listener, from: Origin, between: Between) -> Spot {
    let (at, reach, body, late) = match from {
        Origin::Flat => return Spot::FLAT,
        Origin::At(at, reach) => (at, reach, 0.0, true),
        Origin::Held(at, reach, body) => (at, reach, body, true),
        Origin::Arrived(at, reach) => (at, reach, 0.0, false),
    };
    let to = at - ear.at;
    let metres = to.length();
    let air = if ear.cabin && metres <= HULL.0 {
        Air::home()
    } else {
        ear.air
    };
    let way = Way::of(&air, (ear.wet, between.wet));
    let toward = ear.turn.inverse() * to.normalize_or_zero();
    let side = toward.x * way.width;
    let (lose, dull) = walls(between, ear.cabin, metres);
    Spot {
        air: way.level(metres, reach) * lose,
        body,
        side,
        front: -toward.z,
        delay: if late { metres / way.sound } else { 0.0 },
        corner: way.corner(metres).min(dull),
        gap: around(side, way.ear),
        room: f32::from(u8::from(ear.cabin)),
    }
}
