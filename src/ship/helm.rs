//! The helm: the keyboard turned into what the pilot asks of the ship. The mouse only looks around.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::flight::Controls;
use super::pilot::Autopilot;
use crate::computer::Computer;
use crate::settings::{Keymap, Persistent, Settings};
use crate::space::{Location, Travel};

/// Throttle change per second while a throttle key is held.
const THROTTLE_RATE: f64 = 0.8;

/// What the pilot asks for, kept between frames.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Helm {
    /// Controls handed to the flight step.
    pub controls: Controls,
    /// Whether the warp key was pressed, or the autopilot asks to switch the drive.
    pub warp: bool,
}

/// Input devices and settings the helm reads.
#[derive(SystemParam)]
pub struct Devices<'w> {
    /// Keyboard state.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// Bindings and mouse options.
    settings: Res<'w, Persistent<Settings>>,
    /// Frame time.
    time: Res<'w, Time>,
    /// Whether the board computer is open.
    computer: Option<Res<'w, State<Computer>>>,
}

impl Devices<'_> {
    /// Returns true while the board computer is open: it owns the keyboard and the mouse.
    fn taken(&self) -> bool {
        self.computer
            .as_deref()
            .is_some_and(|state| *state.get() == Computer::Open)
    }

    /// Returns -1, 0 or 1 from a pair of held keys.
    fn axis(&self, positive: KeyCode, negative: KeyCode) -> f64 {
        f64::from(u8::from(self.keys.pressed(positive)))
            - f64::from(u8::from(self.keys.pressed(negative)))
    }

    /// Returns the throttle after this frame: from engines idle to full, never in reverse.
    fn throttle(&self, map: &Keymap, throttle: f64) -> f64 {
        if self.keys.just_pressed(map.ship.cut) {
            return 0.0;
        }
        let push = self.axis(map.ship.throttle_up, map.ship.throttle_down);
        (throttle + push * THROTTLE_RATE * self.time.delta_secs_f64()).clamp(0.0, 1.0)
    }

    /// Returns the controls for the flight step from the keys: the nose goes down and up, left
    /// and right, the hull rolls, the throttle key held at full `throttle` boosts and the
    /// throttle-down key held at idle brakes.
    fn controls(&self, map: &Keymap, throttle: f64) -> Controls {
        let full = throttle >= 1.0;
        let pitch = self.axis(map.ship.pitch_up, map.ship.pitch_down);
        let yaw = self.axis(map.ship.turn_left, map.ship.turn_right);
        let roll = self.axis(map.ship.bank_left, map.ship.bank_right);
        Controls {
            turn: DVec3::new(pitch, yaw, roll),
            lift: self.axis(map.ship.rise, map.ship.sink),
            boost: full && self.keys.pressed(map.ship.throttle_up),
            brake: throttle <= 0.0 && self.keys.pressed(map.ship.throttle_down),
        }
    }
}

/// Reads the helm while the player is seated, and lets go of every control while on foot or
/// while the board computer is open. The throttle lever stays where it was.
pub fn read(
    devices: Devices,
    location: Res<Location>,
    mut helm: ResMut<Helm>,
    mut autopilot: ResMut<Autopilot>,
    mut ship: Single<&mut Ship>,
) {
    if location.travel != Travel::InShip || devices.taken() {
        helm.set_if_neq(Helm::default());
        return;
    }
    let map = devices.settings.controls.keys;
    helm.controls = devices.controls(&map, ship.throttle);
    helm.warp = devices.keys.just_pressed(map.ship.warp);
    if helm.warp {
        autopilot.cancel();
    }
    let throttle = devices.throttle(&map, ship.throttle);
    let assist = ship.assist ^ devices.keys.just_pressed(map.ship.assist);
    if (throttle, assist) != (ship.throttle, ship.assist) {
        (ship.throttle, ship.assist) = (throttle, assist);
    }
}
