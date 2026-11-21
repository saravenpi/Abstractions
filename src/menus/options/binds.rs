//! The rebindable actions: their names, their keys and the conflicts between them.

use bevy::prelude::KeyCode;

use crate::settings::{Keymap, ShipKeys};

/// The name of each action, in the order of [`Bind`].
const LABELS: [&str; 32] = [
    "Forward",
    "Back",
    "Left",
    "Right",
    "Jump",
    "Sprint / boots",
    "Crouch (hold)",
    "Wings (toggle)",
    "Board, leave",
    "Throttle up",
    "Throttle down",
    "Turn left",
    "Turn right",
    "Roll left",
    "Roll right",
    "Thrust up",
    "Thrust down",
    "Nose down",
    "Nose up",
    "Stabiliser",
    "Cut throttle",
    "Warp drive",
    "Switch view",
    "Board computer",
    "Go or stop",
    "Minimap range",
    "Open the map",
    "Drop a pin",
    "Field guide",
    "Pack, workshop",
    "Mend a part",
    "Headlight",
];

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
    Pack,
    Mend,
    Headlight,
}

impl Bind {
    /// The actions used on foot, in display order. The view and board keys work in the ship too,
    /// and the turn keys turn the wingsuit as they turn the ship. The field guide, the pack and the
    /// field repair work on foot only.
    pub const FOOT: [Bind; 18] = [
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
        Bind::Pack,
        Bind::Mend,
    ];

    /// The actions used while seated in the ship, in display order.
    pub const SHIP: [Bind; 21] = [
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
        Bind::Headlight,
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
        LABELS[self as usize]
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
            Bind::Pack => &mut keys.pack,
            Bind::Mend => &mut keys.mend,
            Bind::Computer => &mut keys.computer,
            Bind::Engage => &mut keys.engage,
            ship => ship.ship_slot(&mut keys.ship).expect("a ship key"),
        }
    }

    /// Returns the key of a ship action inside `ship`, or nothing for an action on foot.
    fn ship_slot(self, ship: &mut ShipKeys) -> Option<&mut KeyCode> {
        Some(match self {
            Bind::Board => &mut ship.board,
            Bind::Thrust => &mut ship.throttle_up,
            Bind::Brake => &mut ship.throttle_down,
            Bind::PitchDown => &mut ship.pitch_down,
            Bind::PitchUp => &mut ship.pitch_up,
            Bind::YawLeft => &mut ship.turn_left,
            Bind::YawRight => &mut ship.turn_right,
            Bind::RollLeft => &mut ship.bank_left,
            Bind::RollRight => &mut ship.bank_right,
            Bind::Up => &mut ship.rise,
            Bind::Down => &mut ship.sink,
            Bind::Assist => &mut ship.assist,
            Bind::Cut => &mut ship.cut,
            Bind::Warp => &mut ship.warp,
            Bind::Headlight => &mut ship.headlight,
            _ => return None,
        })
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
