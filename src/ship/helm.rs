//! The helm: keyboard and mouse turned into what the pilot asks of the ship.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseMotion;
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

/// Lowest throttle: reverse thrust.
const REVERSE: f64 = -0.5;

/// Stick deflection per mouse pixel at sensitivity 1.0.
const STICK_PER_PIXEL: f32 = 0.004;

/// Rate per second at which a released mouse stick returns to the centre.
const STICK_RETURN: f32 = 3.0;

/// What the pilot asks for, kept between frames.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Helm {
    /// Controls handed to the flight step.
    pub controls: Controls,
    /// Virtual stick moved by the mouse: x rolls right, y pitches down, each from -1 to 1.
    pub stick: Vec2,
    /// Whether the warp key was pressed, or the autopilot asks to switch the drive.
    pub warp: bool,
}

/// Input devices and settings the helm reads.
#[derive(SystemParam)]
pub struct Devices<'w> {
    /// Keyboard state.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// Mouse motion of the frame.
    mouse: Res<'w, AccumulatedMouseMotion>,
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

    /// Returns the mouse stick after this frame: pushed by the mouse, drifting back to the
    /// centre. While the look key is held the mouse turns the camera and the stick only drifts.
    fn stick(&self, stick: Vec2) -> Vec2 {
        let controls = &self.settings.controls;
        let free = f32::from(u8::from(!self.keys.pressed(controls.keys.orbit)));
        let flip = if controls.invert_y { -1.0 } else { 1.0 };
        let push = self.mouse.delta
            * Vec2::new(1.0, flip)
            * (STICK_PER_PIXEL * controls.mouse_sensitivity * free);
        let eased = stick * (-STICK_RETURN * self.time.delta_secs()).exp();
        (eased + push).clamp_length_max(1.0)
    }

    /// Returns the throttle after this frame.
    fn throttle(&self, map: &Keymap, throttle: f64) -> f64 {
        if self.keys.just_pressed(map.ship.cut) {
            return 0.0;
        }
        let push = self.axis(map.ship.thrust, map.ship.brake);
        (throttle + push * THROTTLE_RATE * self.time.delta_secs_f64()).clamp(REVERSE, 1.0)
    }

    /// Returns the controls for the flight step from the keys and the mouse stick.
    fn controls(&self, map: &Keymap, stick: Vec2) -> Controls {
        let yaw = self.axis(map.ship.yaw_left, map.ship.yaw_right);
        let roll = self.axis(map.ship.roll_left, map.ship.roll_right) - f64::from(stick.x);
        let turn = DVec3::new(f64::from(-stick.y), yaw, roll);
        Controls {
            turn: turn.clamp(DVec3::NEG_ONE, DVec3::ONE),
            lift: self.axis(map.ship.up, map.ship.down),
            boost: self.keys.pressed(map.ship.boost),
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
    helm.stick = devices.stick(helm.stick);
    helm.controls = devices.controls(&map, helm.stick);
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
