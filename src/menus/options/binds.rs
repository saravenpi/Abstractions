//! The rebindable actions: their names, their keys and the conflicts between them.

use bevy::prelude::KeyCode;

use crate::settings::{Keymap, ShipKeys};

/// One rebindable action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bind {
    Forward,
    Back,
    Left,
    Right,
    Jump,
    Sprint,
    Crouch,
    Wings,
    Board,
    Thrust,
    Brake,
    YawLeft,
    YawRight,
    RollLeft,
    RollRight,
    Up,
    Down,
    PitchDown,
    PitchUp,
    Assist,
    Cut,
    Warp,
    View,
    Computer,
    Engage,
    Minimap,
    Map,
    Waypoint,
    Guide,
}

impl Bind {
    /// The actions used on foot, in display order. The view and board keys work in the ship too,
    /// and the turn keys turn the wingsuit as they turn the ship. The field guide opens on foot only.
    pub const FOOT: [Bind; 16] = [
        Bind::Forward,
        Bind::Back,
        Bind::Left,
        Bind::Right,
        Bind::Jump,
        Bind::Sprint,
        Bind::Crouch,
        Bind::Wings,
        Bind::YawLeft,
        Bind::YawRight,
        Bind::View,
        Bind::Board,
        Bind::Minimap,
        Bind::Map,
        Bind::Waypoint,
        Bind::Guide,
    ];

    /// The actions used while seated in the ship, in display order.
    pub const SHIP: [Bind; 20] = [
        Bind::Thrust,
        Bind::Brake,
        Bind::PitchDown,
        Bind::PitchUp,
        Bind::YawLeft,
        Bind::YawRight,
        Bind::RollLeft,
        Bind::RollRight,
        Bind::Up,
        Bind::Down,
        Bind::Assist,
        Bind::Cut,
        Bind::Warp,
        Bind::View,
        Bind::Board,
        Bind::Computer,
        Bind::Engage,
        Bind::Minimap,
        Bind::Map,
        Bind::Waypoint,
    ];

    /// Returns the name of the action.
    pub fn label(self) -> &'static str {
        match self {
            Bind::Forward => "Forward",
            Bind::Back => "Back",
            Bind::Left => "Left",
            Bind::Right => "Right",
            Bind::Jump => "Jump",
            Bind::Sprint => "Sprint / boots",
            Bind::Crouch => "Crouch (hold)",
            Bind::Wings => "Wings (toggle)",
            Bind::View => "Switch view",
            Bind::Minimap => "Minimap",
            Bind::Map => "Map",
            Bind::Waypoint => "Waypoint",
            Bind::Guide => "Field guide",
            _ => self.ship_label(),
        }
    }

    /// Returns the name of an action of the ship.
    fn ship_label(self) -> &'static str {
        match self {
            Bind::Board => "Board or leave ship",
            Bind::Thrust => "Throttle up, boost",
            Bind::Brake => "Throttle, brake",
            Bind::YawLeft => "Turn left",
            Bind::YawRight => "Turn right",
            Bind::RollLeft => "Roll left",
            Bind::RollRight => "Roll right",
            Bind::Up => "Thrust up",
            Bind::Down => "Thrust down",
            Bind::PitchDown => "Nose down",
            Bind::PitchUp => "Nose up",
            Bind::Assist => "Flight assist",
            Bind::Cut => "Cut throttle",
            Bind::Computer => "Board computer",
            Bind::Engage => "Engage autopilot",
            _ => "Warp drive",
        }
    }

    /// Returns the key of the action inside `keys`, for reading or writing.
    pub fn slot(self, keys: &mut Keymap) -> &mut KeyCode {
        match self {
            Bind::Forward => &mut keys.forward,
            Bind::Back => &mut keys.back,
            Bind::Left => &mut keys.left,
            Bind::Right => &mut keys.right,
            Bind::Jump => &mut keys.jump,
            Bind::Sprint => &mut keys.sprint,
            Bind::Crouch => &mut keys.crouch,
            Bind::Wings => &mut keys.wings,
            Bind::View => &mut keys.view,
            Bind::Minimap => &mut keys.minimap,
            Bind::Map => &mut keys.map,
            Bind::Waypoint => &mut keys.waypoint,
            Bind::Guide => &mut keys.guide,
            Bind::Computer => &mut keys.computer,
            Bind::Engage => &mut keys.engage,
            _ => self.ship_slot(&mut keys.ship),
        }
    }

    /// Returns the key of an action of the ship inside `keys`.
    fn ship_slot(self, keys: &mut ShipKeys) -> &mut KeyCode {
        match self {
            Bind::Board => &mut keys.board,
            Bind::Thrust => &mut keys.throttle_up,
            Bind::Brake => &mut keys.throttle_down,
            Bind::PitchDown => &mut keys.pitch_down,
            Bind::PitchUp => &mut keys.pitch_up,
            Bind::YawLeft => &mut keys.turn_left,
            Bind::YawRight => &mut keys.turn_right,
            Bind::RollLeft => &mut keys.bank_left,
            Bind::RollRight => &mut keys.bank_right,
            Bind::Up => &mut keys.rise,
            Bind::Down => &mut keys.sink,
            Bind::Assist => &mut keys.assist,
            Bind::Cut => &mut keys.cut,
            _ => &mut keys.warp,
        }
    }

    /// Returns the key bound to the action.
    pub fn key(self, keys: &Keymap) -> KeyCode {
        let mut copy = *keys;
        *self.slot(&mut copy)
    }
}

/// Returns every pair of actions that share a key while both are in use: two actions on foot,
/// or two actions in the ship. An action on foot and one in the ship may share a key.
pub fn conflicts(keys: &Keymap) -> Vec<(Bind, Bind)> {
    let mut pairs = Vec::new();
    for group in [&Bind::FOOT[..], &Bind::SHIP[..]] {
        for (at, &first) in group.iter().enumerate() {
            let same = |other: &&Bind| other.key(keys) == first.key(keys);
            pairs.extend(
                group[at + 1..]
                    .iter()
                    .filter(same)
                    .map(|&other| (first, other)),
            );
        }
    }
    pairs
}

/// Returns true when `bind` shares its key with another action of the same group.
pub fn conflicted(keys: &Keymap, bind: Bind) -> bool {
    conflicts(keys)
        .iter()
        .any(|(first, second)| *first == bind || *second == bind)
}
