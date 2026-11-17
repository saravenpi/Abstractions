//! The lights of the base: what each one looks like at a time of day and when the robot is near.

use std::f32::consts::TAU;

use bevy::prelude::*;

/// Seconds between two double flashes of the beacon, and between two pulses of the antenna.
const PERIODS: (f32, f32) = (3.0, 4.5);

/// Seconds a flash of the beacon lasts.
const FLASH: f32 = 0.14;

/// Share of its night power a light keeps at full day.
const DAY_SHARE: f32 = 0.15;

/// One glowing part of the base. Each is its own mesh and material.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glow {
    /// Thin status strip above the hatch.
    Strip,
    /// Tinted window slit of the container.
    Window,
    /// Light at the tip of the antenna, pulsing slowly with the quantum link.
    Pulse,
    /// Soft light over the charging dock.
    Dock,
    /// Beacon on the mast of the station.
    Beacon,
    /// The four corner lights of the landing pad.
    Corner,
}

/// What the lights answer to.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mood {
    /// Seconds since the game started.
    pub seconds: f32,
    /// How dark it is, from 0 at full day to 1 at night.
    pub night: f32,
    /// How much the operator has noticed the robot at the hatch, from 0 to 1.
    pub notice: f32,
}

impl Glow {
    /// Every glow, in the order of `self as usize`.
    pub const ALL: [Glow; 6] = [
        Glow::Strip,
        Glow::Window,
        Glow::Pulse,
        Glow::Dock,
        Glow::Beacon,
        Glow::Corner,
    ];

    /// Returns the linear colour of the glow at full strength.
    fn tint(self, notice: f32) -> Vec3 {
        match self {
            Glow::Strip => Vec3::new(0.25, 0.95, 0.8),
            Glow::Window => Vec3::new(1.0, 0.66, 0.3).lerp(Vec3::new(1.0, 0.86, 0.62), notice),
            Glow::Pulse => Vec3::new(0.45, 0.7, 1.0),
            Glow::Dock => Vec3::new(1.0, 0.9, 0.74),
            Glow::Beacon => Vec3::new(1.0, 0.74, 0.38),
            Glow::Corner => Vec3::new(0.3, 0.62, 1.0),
        }
    }

    /// Returns the rhythm of the glow at `seconds`, around one: the flashes of the beacon, the
    /// slow pulse of the antenna and the faint breathing of the window.
    fn beat(self, seconds: f32) -> f32 {
        let wave = |period: f32| 0.5 + 0.5 * (seconds * TAU / period).sin();
        match self {
            Glow::Window => 0.94 + 0.12 * wave(9.0),
            Glow::Pulse => 0.25 + 3.0 * wave(PERIODS.1).powi(3),
            Glow::Beacon => 0.12 + 0.88 * flash(seconds),
            _ => 1.0,
        }
    }

    /// Returns how many times white the glow shows: above one it blooms. Dim by day, full at night.
    pub fn gain(self, mood: Mood) -> f32 {
        let (day, night, notice) = match self {
            Glow::Strip => (1.1, 2.4, 1.6),
            Glow::Window => (0.9, 1.6, 0.3),
            Glow::Pulse => (0.8, 1.0, 0.0),
            Glow::Dock => (1.0, 2.0, 0.0),
            Glow::Beacon => (2.5, 10.0, 0.0),
            Glow::Corner => (0.9, 1.8, 0.0),
        };
        let lit = day + (night - day) * mood.night.clamp(0.0, 1.0);
        lit * self.beat(mood.seconds) * (1.0 + notice * mood.notice)
    }

    /// Returns the unlit colour of the glow, brighter than white where it must bloom.
    pub fn color(self, mood: Mood) -> Color {
        let rgb = self.tint(mood.notice) * self.gain(mood);
        Color::linear_rgb(rgb.x, rgb.y, rgb.z)
    }

    /// Returns the colour of the light the glow casts, at full strength.
    pub fn cast(self) -> Color {
        let rgb = self.tint(0.0);
        Color::linear_rgb(rgb.x, rgb.y, rgb.z)
    }

    /// Returns the power in lumens at night and the range in metres of the light the glow
    /// casts, or nothing for a glow that lights nothing around it.
    pub fn lamp(self) -> Option<(f32, f32)> {
        match self {
            Glow::Window => Some((5_000.0, 5.0)),
            Glow::Dock => Some((14_000.0, 7.0)),
            Glow::Beacon => Some((60_000.0, 30.0)),
            _ => None,
        }
    }

    /// Returns the power in lumens of the light the glow casts in `mood`: weak by day, and in
    /// step with the glow.
    pub fn power(self, mood: Mood) -> f32 {
        let share = mood.night.clamp(0.0, 1.0).powi(2).max(DAY_SHARE);
        let lumens = self.lamp().map_or(0.0, |(lumens, _)| lumens);
        lumens * share * self.beat(mood.seconds)
    }
}

/// Returns the strength of the beacon at `seconds`, from 0 to 1: two short flashes per period.
pub fn flash(seconds: f32) -> f32 {
    let at = seconds.rem_euclid(PERIODS.0);
    let bump = |start: f32| (1.0 - ((at - start) / FLASH - 1.0).powi(2)).max(0.0);
    bump(0.1).max(bump(0.5))
}
