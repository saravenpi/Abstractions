//! The helm: the keyboard turned into what the pilot asks of the ship, and the mouse too while
//! its left button is held. With the button up the mouse only looks around.

pub mod mouse;

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::assist::{eased, ramp};
use super::body::Ship;
use super::flight::Controls;
use super::pilot::Autopilot;
use super::warp::Warp;
use crate::computer::Computer;
use crate::settings::{Keymap, Persistent, Settings};
use crate::space::{Frame, Location, Travel};
use mouse::{Mouse, deflection};

/// Throttle change per second while a throttle key is held.
const THROTTLE_RATE: f64 = 0.8;

/// What the pilot asks for, kept between frames.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Helm {
    /// Controls handed to the flight step.
    pub controls: Controls,
    /// Whether the warp key was pressed, or the autopilot asks to switch the drive.
    pub warp: bool,
    /// The point beside a body in the way that the autopilot flies the leg of its route to, in
    /// the axes of the frame of the ship. Nothing when the leg goes to the destination.
    pub way: Option<DVec3>,
    /// How far each turn key has been held, from -1 to 1: pitch, yaw and roll.
    pub held: DVec3,
    /// The stick the mouse moves while its left button is held, x to the right and y downward,
    /// each from -1 to 1. It is centred while the button is up.
    pub stick: Vec2,
}

/// Input devices and settings the helm reads.
#[derive(SystemParam)]
pub struct Devices<'w> {
    /// Keyboard state.
    keys: Res<'w, ButtonInput<KeyCode>>,
    /// The mouse, which steers while its left button is held.
    mouse: Mouse<'w>,
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

    /// Returns the turn keys held: nose up and down, left and right, and roll, -1, 0 or 1 each.
    fn turn(&self, map: &Keymap) -> DVec3 {
        DVec3::new(
            self.axis(map.ship.pitch_up, map.ship.pitch_down),
            self.axis(map.ship.turn_left, map.ship.turn_right),
            self.axis(map.ship.bank_left, map.ship.bank_right),
        )
    }

    /// Returns the controls for the flight step from the keys: the stick `turn`, the throttle
    /// key held at full `throttle` boosts and the throttle-down key held at idle brakes.
    fn controls(&self, map: &Keymap, throttle: f64, turn: DVec3) -> Controls {
        let full = throttle >= 1.0;
        Controls {
            turn,
            lift: self.axis(map.ship.rise, map.ship.sink),
            boost: full && self.keys.pressed(map.ship.throttle_up),
            brake: throttle <= 0.0 && self.keys.pressed(map.ship.throttle_down),
        }
    }
}

/// Returns the turn the keys or the mouse ask of a ship whose drive is in `warp`: nothing while
/// the warp is engaged, when the ship flies the line it was pointed along and no hand could
/// steer it, and what they ask otherwise, the spool included, so the pilot can still line up.
pub fn steerable(warp: Warp, asked: DVec3) -> DVec3 {
    match warp {
        Warp::Active { .. } => DVec3::ZERO,
        _ => asked,
    }
}

/// Reads the helm while the player is seated, and lets go of every control while on foot or
/// while the board computer is open. The throttle lever stays where it was. With the assist on
/// over a planet the turn keys ease in over [`RAMP`](super::assist::RAMP), so that a tap is a
/// small correction; without it, and in space, they act at once. The mouse adds its stick to
/// the keys while its left button is held, and nothing while it is up.
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
    let keys = steerable(ship.warp, devices.turn(&map));
    helm.held = ramp(helm.held, keys, devices.time.delta_secs_f64());
    let surface = matches!(ship.frame, Frame::Surface(_));
    let stick = if ship.assist && surface {
        eased(helm.held)
    } else {
        keys
    };
    let controls = &devices.settings.controls;
    helm.stick = devices
        .mouse
        .stick(helm.stick, controls, devices.time.delta_secs());
    let both = (stick + deflection(helm.stick)).clamp(DVec3::NEG_ONE, DVec3::ONE);
    let turn = steerable(ship.warp, both);
    helm.controls = devices.controls(&map, ship.throttle, turn);
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
